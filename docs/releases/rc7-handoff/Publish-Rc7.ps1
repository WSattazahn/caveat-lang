# Generated release-specific constants must be filled only from final-main receipts.
[CmdletBinding()]
param([switch]$VerifyOnly)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$Registry = 'https://registry.npmjs.org'
$Version = '0.1.0-rc.7'
$Revision = '1f3fc7a2208eec964399d6c14232690f411e48be'
$ExpectedSha256 = '0e441f896c58ba41f2741a82af40a17e74247a0c765169abda10b9ddffad5bbc'
$ExpectedIntegrity = 'sha512-CyxtunAuNyo1k7KLQhovEvMwrS+uWP2U4VLTukZTQwNRcKJbvI8E5AfYXGgd8C4tOC1oSixH3YD2J5El7Lim8g=='
$ExpectedWasm = '32139e7b59306e4da799f6977ec9356f68add27c7f7159a443d7a1f56cae8977'
$Asset = 'https://github.com/WSattazahn/caveat-lang/releases/download/v0.1.0-rc.7/caveat-lang-0.1.0-rc.7.tgz'
if ($ExpectedSha256 -notmatch '^[a-f0-9]{64}$') { throw 'Release freeze has not completed. Do not publish.' }
$Gh = (Get-Command gh.exe -ErrorAction Stop).Source
& $Gh auth status --hostname github.com
if ($LASTEXITCODE -ne 0) {
    & $Gh auth login --hostname github.com --git-protocol https --web
    if ($LASTEXITCODE -ne 0) { throw 'GitHub login failed; nothing published' }
}
$Npm = (Get-Command npm.cmd -ErrorAction Stop).Source
$Node = (Get-Command node.exe -ErrorAction Stop).Source
function Invoke-Native([string]$Exe, [string[]]$Arguments) {
    & $Exe @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Exe exited $LASTEXITCODE" }
}
function Read-NativeJson([string]$Exe, [string[]]$Arguments) {
    $output = & $Exe @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Exe exited $LASTEXITCODE" }
    return ($output -join "`n" | ConvertFrom-Json)
}
function Assert-Hash([string]$File) {
    if ((Get-FileHash -LiteralPath $File -Algorithm SHA256).Hash.ToLowerInvariant() -ne $ExpectedSha256) { throw 'Tarball SHA256 mismatch' }
    $sha = [Security.Cryptography.SHA512]::Create()
    try { $integrity = 'sha512-' + [Convert]::ToBase64String($sha.ComputeHash([IO.File]::ReadAllBytes($File))) } finally { $sha.Dispose() }
    if ($integrity -ne $ExpectedIntegrity) { throw 'Tarball SHA512 integrity mismatch' }
}
function Get-RegistryVersion {
    try { return (Invoke-RestMethod -Uri "$Registry/caveat-lang/$Version") }
    catch {
        $response = $_.Exception.Response
        if ($null -ne $response -and [int]$response.StatusCode -eq 404) { return $null }
        throw
    }
}
$Release = Read-NativeJson -Exe $Gh -Arguments @('api','repos/WSattazahn/caveat-lang/releases/tags/v0.1.0-rc.7')
if (-not $Release.prerelease -or $Release.draft) { throw 'GitHub release must remain a published prerelease' }
$Ref = Read-NativeJson -Exe $Gh -Arguments @('api','repos/WSattazahn/caveat-lang/git/ref/tags/v0.1.0-rc.7')
if ($Ref.object.type -ne 'tag') { throw 'GitHub release tag is not annotated' }
$Tag = Read-NativeJson -Exe $Gh -Arguments @('api',('repos/WSattazahn/caveat-lang/git/tags/' + $Ref.object.sha))
if ($Tag.object.type -ne 'commit' -or $Tag.object.sha -ne $Revision) { throw 'GitHub tag revision mismatch' }
$Run = Join-Path ([IO.Path]::GetTempPath()) ('caveat-rc7-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Run | Out-Null
$Tarball = Join-Path $Run 'caveat-lang-0.1.0-rc.7.tgz'
Invoke-WebRequest -UseBasicParsing -Uri $Asset -OutFile $Tarball
Assert-Hash $Tarball
$NodeVersion = & $Node --version
if ($LASTEXITCODE -ne 0) { throw 'Node version command failed' }
$NodeMajor = [int]($NodeVersion.TrimStart('v').Split('.')[0])
if ($LASTEXITCODE -ne 0 -or $NodeMajor -lt 20) { throw 'Node 20 or later is required' }
function Test-Consumer([string]$Package, [string]$Name) {
    $Consumer = Join-Path $Run $Name
    New-Item -ItemType Directory -Path $Consumer | Out-Null
    Push-Location $Consumer
    try {
        Invoke-Native -Exe $Npm -Arguments @('init','-y')
        Invoke-Native -Exe $Npm -Arguments @('install',$Package,'--ignore-scripts','--no-audit','--no-fund','--registry',$Registry,'--cache',(Join-Path $Consumer 'cache'))
        if ($Name -eq 'registry-install') {
            $Lock = Get-Content -LiteralPath 'package-lock.json' -Raw | ConvertFrom-Json
            $Locked = $Lock.packages.'node_modules/caveat-lang'
            if ($Locked.integrity -ne $ExpectedIntegrity -or ([Uri]$Locked.resolved).Host -ne 'registry.npmjs.org') { throw 'Fresh-install lock integrity/origin mismatch' }
        }
        $PackageRoot = Join-Path $Consumer 'node_modules/caveat-lang'
        $Manifest = Get-Content -LiteralPath (Join-Path $PackageRoot 'package.json') -Raw | ConvertFrom-Json
        if ($Manifest.name -ne 'caveat-lang' -or $Manifest.version -ne $Version) { throw 'Installed identity mismatch' }
        foreach ($Alias in @('caveat-lang','caveat')) {
            $Cli = Join-Path $Consumer "node_modules/.bin/$Alias.cmd"
            $Reported = & $Cli --version
            if ($LASTEXITCODE -ne 0 -or ($Reported -join "`n") -ne "CAVEAT Language $Version") { throw 'CLI identity mismatch' }
        }
        $Cli = Join-Path $Consumer 'node_modules/.bin/caveat-lang.cmd'
        $Doctor = Read-NativeJson -Exe $Cli -Arguments @('doctor','--json')
        $Build = $Doctor.runtime.buildInfo
        if (-not $Doctor.ok -or $Doctor.package.version -ne $Version -or $Build.revision -ne $Revision -or -not $Build.clean -or -not $Build.compiled -or $Build.host -ne 'x86_64-unknown-linux-gnu') { throw 'Doctor/build identity mismatch' }
        $Wasm = Join-Path $PackageRoot 'runtime/caveat_runtime_bg.wasm'
        if ((Get-FileHash -LiteralPath $Wasm -Algorithm SHA256).Hash.ToLowerInvariant() -ne $ExpectedWasm) { throw 'Installed WASM mismatch' }
        $Demo = Read-NativeJson -Exe $Cli -Arguments @('demo','agent','--json')
        if (-not $Demo.preservation.unchanged -or $Demo.steps.Count -ne 5) { throw 'Agent demo failed' }
        Invoke-Native -Exe $Cli -Arguments @('init')
        Invoke-Native -Exe $Cli -Arguments @('test','umbrella.scenarios.json')
        Invoke-Native -Exe $Cli -Arguments @('explain','umbrella.cav','events.jsonl')
    } finally { Pop-Location }
}
Test-Consumer $Tarball 'before-publication'
$Tags = Invoke-RestMethod -Uri "$Registry/-/package/caveat-lang/dist-tags"
if ($Tags.latest -ne '0.1.0-rc.5' -or $Tags.next -notin @('0.1.0-rc.6',$Version)) { throw 'Unexpected dist-tags; stop for review' }
$Existing = Get-RegistryVersion
if ($null -eq $Existing) {
    if ($VerifyOnly) { throw 'rc.7 is not published' }
    Write-Host 'npm login may open a browser and request account/2FA approval. No credentials are recorded.'
    Invoke-Native -Exe $Npm -Arguments @('login','--registry',$Registry)
    Invoke-Native -Exe $Npm -Arguments @('publish',$Tarball,'--access','public','--tag','next','--ignore-scripts','--registry',$Registry)
} else {
    if ($Existing.dist.integrity -ne $ExpectedIntegrity -or $Existing.name -ne 'caveat-lang' -or $Existing.version -ne $Version) { throw 'Existing version contains different bytes or identity' }
    Write-Host 'Exact version exists; publication skipped. Verifying registry bytes.'
}
$Published = Get-RegistryVersion
if ($null -eq $Published -or $Published.dist.integrity -ne $ExpectedIntegrity -or $Published.name -ne 'caveat-lang' -or $Published.version -ne $Version) { throw 'Registry identity/integrity mismatch' }
$DownloadUri = [Uri]$Published.dist.tarball
if ($DownloadUri.Scheme -ne 'https' -or $DownloadUri.Host -ne 'registry.npmjs.org') { throw 'Unexpected registry tarball origin' }
$RegistryTarball = Join-Path $Run 'registry-rc7.tgz'
Invoke-WebRequest -UseBasicParsing -Uri $DownloadUri -OutFile $RegistryTarball
Assert-Hash $RegistryTarball
Test-Consumer "caveat-lang@$Version" 'registry-install'
$Tags = Invoke-RestMethod -Uri "$Registry/-/package/caveat-lang/dist-tags"
if ($Tags.next -ne $Version -or $Tags.latest -ne '0.1.0-rc.5') { throw 'Unexpected channels after publication; explicit owner review is required. No dist-tag was silently changed.' }
$NpmVersion = & $Npm --version
if ($LASTEXITCODE -ne 0) { throw 'npm version command failed' }
$Receipt = Join-Path $Run 'npm-publication-verification.json'
@{version=$Version; revision=$Revision; sha256=$ExpectedSha256; integrity=$ExpectedIntegrity; registry=$Registry; verifiedAt=[DateTime]::UtcNow.ToString('o'); next=$Tags.next; latest=$Tags.latest; exactVersionFreshInstall=$true; nodeVersion=$NodeVersion; npmVersion=$NpmVersion} | ConvertTo-Json | Set-Content -LiteralPath $Receipt -Encoding UTF8
Write-Host "Verified exact rc.7 publication. Receipt: $Receipt"
$Gh = (Get-Command gh.exe -ErrorAction Stop).Source
Invoke-Native -Exe $Gh -Arguments @('auth','status','--hostname','github.com')
$PublicReceipt = Join-Path $Run ('npm-publication-verification-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfff') + '.json')
Copy-Item -LiteralPath $Receipt -Destination $PublicReceipt
Invoke-Native -Exe $Gh -Arguments @('release','upload','v0.1.0-rc.7',$PublicReceipt,'--repo','WSattazahn/caveat-lang')
$Marker = '## Verified npm publication'
if ($Release.body -notmatch [Regex]::Escape($Marker)) {
    $Notes = Join-Path $Run 'github-published-notes.md'
    $Addition = "`n`n$Marker`n`nOfficial npm version ``caveat-lang@$Version`` verified at $([DateTime]::UtcNow.ToString('o')). Registry integrity and downloaded bytes match the retained tested tgz (SHA256 ``$ExpectedSha256``). A fresh exact-version install passes both CLI identities, doctor/demo and umbrella scenarios. ``next`` names rc.7; ``latest`` remains rc.5. The attached publication verification receipt records this owner-console check. Documentation publication records are the explicit cloud follow-up; rc.8 development is already open.`n"
    [IO.File]::WriteAllText($Notes,(([string]$Release.body).Replace('GitHub candidate available; npm publication pending.','GitHub candidate and exact npm version available; npm publication verified below.') + $Addition),[Text.UTF8Encoding]::new($false))
    Invoke-Native -Exe $Gh -Arguments @('release','edit','v0.1.0-rc.7','--repo','WSattazahn/caveat-lang','--notes-file',$Notes,'--prerelease','--latest=false')
}
Write-Host 'GitHub publication record updated only after registry/fresh-install verification. Return the receipt to the cloud thread for public install-doc updates. latest was not changed.'
