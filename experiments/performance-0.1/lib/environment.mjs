// The machine, toolchain and load a run was measured under.
import { spawn, spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';

function output(command, args, cwd, shell = false) {
  const result = spawnSync(command, args, { cwd, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024, shell });
  return result.status === 0 ? result.stdout.trim() : null;
}

// Windows: CPU, cores and each logical processor's CPU set (logical
// processor, core, efficiency class, scheduling class, last-level cache; P-cores
// have the higher efficiency class on hybrid Intel parts), memory, OS build,
// power plan and whether the machine is on mains power.
const WINDOWS_MACHINE = String.raw`
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class CaveatPerfCpuSets {
  [DllImport("kernel32.dll", SetLastError = true)]
  static extern bool GetSystemCpuSetInformation(IntPtr info, uint length, out uint returned, IntPtr process, uint flags);
  public static string Read() {
    uint needed;
    GetSystemCpuSetInformation(IntPtr.Zero, 0, out needed, IntPtr.Zero, 0);
    IntPtr buffer = Marshal.AllocHGlobal((int)needed);
    try {
      if (!GetSystemCpuSetInformation(buffer, needed, out needed, IntPtr.Zero, 0)) return "";
      var parts = new System.Collections.Generic.List<string>();
      int offset = 0;
      while (offset < needed) {
        int size = Marshal.ReadInt32(buffer, offset);
        if (Marshal.ReadInt32(buffer, offset + 4) == 0) {
          parts.Add(Marshal.ReadByte(buffer, offset + 14) + ":" + Marshal.ReadByte(buffer, offset + 15) + ":" + Marshal.ReadByte(buffer, offset + 18) + ":" + Marshal.ReadByte(buffer, offset + 20) + ":" + Marshal.ReadByte(buffer, offset + 16));
        }
        offset += size;
      }
      return String.Join(",", parts);
    } finally { Marshal.FreeHGlobal(buffer); }
  }
}
"@
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$os = Get-CimInstance Win32_OperatingSystem
$battery = Get-CimInstance Win32_Battery -ErrorAction SilentlyContinue | Select-Object -First 1
[pscustomobject]@{
  cpu = $cpu.Name.Trim(); cores = $cpu.NumberOfCores; logicalProcessors = $cpu.NumberOfLogicalProcessors; maxClockMHz = $cpu.MaxClockSpeed
  os = $os.Caption; osVersion = $os.Version; osBuild = $os.BuildNumber; ramBytes = [int64]$os.TotalVisibleMemorySize * 1024
  powerScheme = ((powercfg /getactivescheme) -join ' ').Trim()
  batteryStatus = if ($battery) { $battery.BatteryStatus } else { $null }
  cpuSets = [CaveatPerfCpuSets]::Read()
} | ConvertTo-Json -Compress
`;

// Windows: total CPU use sampled over a few seconds, and the processes that
// used the most CPU time in that window.
const WINDOWS_LOAD = String.raw`
$ErrorActionPreference = 'Stop'
$before = @{}; Get-Process | ForEach-Object { if ($_.CPU -ne $null) { $before[$_.Id] = $_.CPU } }
$samples = @()
for ($i = 0; $i -lt 3; $i++) {
  Start-Sleep -Milliseconds 1000
  $samples += (Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor -Filter "Name='_Total'").PercentProcessorTime
}
$top = Get-Process | Where-Object { $_.CPU -ne $null -and $before.ContainsKey($_.Id) } |
  ForEach-Object { [pscustomobject]@{ name = $_.ProcessName; pid = $_.Id; cpuSeconds = [math]::Round($_.CPU - $before[$_.Id], 3) } } |
  Sort-Object cpuSeconds -Descending | Select-Object -First 6
[pscustomobject]@{ totalCpuPercent = $samples; busiestProcesses = $top } | ConvertTo-Json -Compress -Depth 3
`;

async function powershell(script, scratch, name) {
  const file = path.join(scratch, `${name}.ps1`);
  await writeFile(file, script);
  const text = output('powershell', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', file]);
  try { return text ? JSON.parse(text) : null; } catch { return { unparsed: text }; }
}

// cpuSets "logical:core:efficiency[:scheduling:cache],..." to the logical
// processors of the highest efficiency class (the performance cores on a
// hybrid part).
export function performanceCores(cpuSets) {
  if (!cpuSets) return null;
  const sets = cpuSets.split(',').map((entry) => entry.split(':').map(Number));
  const best = Math.max(...sets.map(([, , efficiency]) => efficiency));
  return sets.filter(([, , efficiency]) => efficiency === best).map(([logical]) => logical);
}

// The same text, one record per logical processor. Older records have only
// the first three fields.
export function parseCpuSets(cpuSets) {
  if (!cpuSets) return null;
  return cpuSets.split(',').map((entry) => {
    const [logicalProcessor, core, efficiencyClass, schedulingClass, lastLevelCache] = entry.split(':').map(Number);
    return { logicalProcessor, core, efficiencyClass, schedulingClass: schedulingClass ?? null, lastLevelCache: lastLevelCache ?? null };
  });
}

export async function machine(scratch) {
  const info = {
    platform: process.platform,
    arch: process.arch,
    release: os.release(),
    cpuModel: os.cpus()[0]?.model?.trim() ?? null,
    logicalProcessors: os.cpus().length,
    totalMemoryBytes: os.totalmem(),
  };
  if (process.platform === 'win32') {
    info.windows = await powershell(WINDOWS_MACHINE, scratch, 'machine');
    info.performanceCores = performanceCores(info.windows?.cpuSets);
    info.cpuSetList = parseCpuSets(info.windows?.cpuSets);
  }
  return info;
}

export async function load(scratch) {
  if (process.platform === 'win32') return powershell(WINDOWS_LOAD, scratch, 'load');
  return { loadavg: os.loadavg() };
}

// Windows: samples total CPU use, and each pinned logical processor's use and
// actual clock (MHz), every `seconds` for the whole run with typeperf (one
// sample line per interval, read from its output). The benchmark itself
// keeps about one pinned processor busy, so the pinned processors' sum above
// 100% and the total above one processor's share (100 / logical processors)
// is other load; the busy pinned processor is the one the benchmark ran on.
export function startMonitor(processors = [], seconds = 5) {
  if (process.platform !== 'win32') return null;
  const counters = [
    '\\Processor(_Total)\\% Processor Time',
    ...processors.map((index) => `\\Processor(${index})\\% Processor Time`),
    ...processors.map((index) => `\\Processor Information(0,${index})\\Actual Frequency`),
  ];
  const child = spawn('typeperf', [...counters, '-si', String(seconds)], { stdio: ['ignore', 'pipe', 'ignore'] });
  let text = '';
  child.stdout.on('data', (chunk) => { text += chunk; });
  return {
    seconds,
    processors,
    counters: ['total', ...processors.map((index) => `processor ${index}`), ...processors.map((index) => `processor ${index} MHz`)],
    stop() {
      child.kill();
      const rows = text.split(/\r?\n/).filter((line) => /^"\d/.test(line))
        .map((line) => line.split('","').map((cell) => cell.replace(/"/g, '')));
      return { csv: text.trim(), rows };
    },
  };
}

// Mean, p95 and max of each monitored counter, and how many samples had
// total CPU use above 10% and 25%.
export function summarizeMonitor(monitor, stopped) {
  if (!monitor || !stopped) return null;
  const columns = monitor.counters.map((name, index) => {
    const values = stopped.rows.map((row) => Number(row[index + 1])).filter(Number.isFinite).sort((a, b) => a - b);
    const mean = values.reduce((sum, value) => sum + value, 0) / (values.length || 1);
    return [name, { samples: values.length, mean: Number(mean.toFixed(1)), p95: values[Math.floor(0.95 * (values.length - 1))] ?? null, max: values.at(-1) ?? null }];
  });
  const totals = stopped.rows.map((row) => Number(row[1])).filter(Number.isFinite);
  const busyColumns = (monitor.processors ?? []).length;
  const pinned = stopped.rows.map((row) => row.slice(2, 2 + busyColumns).map(Number).reduce((sum, value) => sum + value, 0));
  return {
    what: `typeperf every ${monitor.seconds} s for the whole run`,
    samples: totals.length,
    counters: Object.fromEntries(columns.map(([name, stats]) => [name, { ...stats, p95: stats.p95 === null ? null : Number(stats.p95.toFixed(1)), max: stats.max === null ? null : Number(stats.max.toFixed(1)) }])),
    totalAbove10Percent: totals.filter((value) => value > 10).length,
    totalAbove25Percent: totals.filter((value) => value > 25).length,
    pinnedSumAbove150Percent: busyColumns > 0 ? pinned.filter((value) => value > 150).length : null,
  };
}

// Which monitored processors a job ran on, from the samples taken while it
// ran (typeperf stamps local time): each processor's mean use and its mean
// actual clock over the samples where it was more than half busy.
export function coresDuring(monitor, stopped, fromMs, toMs) {
  if (!monitor || !stopped || !(monitor.processors ?? []).length) return null;
  const count = monitor.processors.length;
  const rows = stopped.rows.filter((row) => {
    const at = new Date(row[0]).getTime();
    return Number.isFinite(at) && at >= fromMs && at <= toMs;
  });
  if (!rows.length) return { samples: 0 };
  const mean = (values) => (values.length ? Number((values.reduce((a, b) => a + b, 0) / values.length).toFixed(1)) : null);
  const cores = {};
  monitor.processors.forEach((processor, index) => {
    const busy = rows.map((row) => Number(row[2 + index]));
    const clock = rows.map((row) => Number(row[2 + count + index]));
    const hot = clock.filter((value, at) => busy[at] > 50 && Number.isFinite(value));
    cores[processor] = { busyPercent: mean(busy.filter(Number.isFinite)), mhzWhileBusy: mean(hot), busySamples: hot.length };
  });
  return { samples: rows.length, cores };
}

// Toolchain as a given tree selects it (rust-toolchain.toml pins rustc).
export function toolchain(tree) {
  return {
    node: process.version,
    v8: process.versions.v8,
    npm: output('npm --version', [], tree, true),
    rustc: tree ? output('rustc', ['-vV'], tree) : null,
    cargo: tree ? output('cargo', ['--version'], tree) : null,
    rustupToolchain: tree ? output('rustup', ['show', 'active-toolchain'], tree) : null,
    wasmBindgen: output('wasm-bindgen', ['--version'], tree),
    wasmOpt: output('wasm-opt', ['--version'], tree),
  };
}
