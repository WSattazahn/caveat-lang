$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$Path = Join-Path $PSScriptRoot '../docs/releases/rc7-handoff/Publish-Rc7.ps1'
$Tokens = $null; $Errors = $null
$Ast = [Management.Automation.Language.Parser]::ParseFile((Resolve-Path $Path),[ref]$Tokens,[ref]$Errors)
if ($Errors.Count) { throw ($Errors | Out-String) }
# Load only declarations: never execute authentication/publication/main helper.
foreach ($Name in @('Read-NativeJson','Get-RegistryVersion','Assert-RegistryIdentity','Wait-RegistryPublication','Invoke-PublicationDecision','Read-InstalledLock')) {
    $Function = $Ast.Find({param($Node) $Node -is [Management.Automation.Language.FunctionDefinitionAst] -and $Node.Name -eq $Name},$true)
    if ($null -eq $Function) { throw "Missing function $Name" }
    Invoke-Expression $Function.Extent.Text
}
$Version = '0.1.0-rc.7'; $ExpectedIntegrity = 'expected'
$Valid = [pscustomobject]@{name='caveat-lang';version=$Version;dist=[pscustomobject]@{integrity=$ExpectedIntegrity}}
$GoodTags = [pscustomobject]@{next=$Version;latest='0.1.0-rc.5'}
$OldTags = [pscustomobject]@{next='0.1.0-rc.6';latest='0.1.0-rc.5'}
function Assert($Condition,$Message) { if (-not $Condition) { throw $Message } }
function Refuses([scriptblock]$Action,[string]$Pattern) {
    try { & $Action | Out-Null } catch { if ($_.Exception.Message -notmatch $Pattern) { throw }; return }
    throw "Expected refusal: $Pattern"
}
$script:Reads=0; $script:Delays=0
$Result=Wait-RegistryPublication -ReadVersion { $script:Reads++; if ($script:Reads -gt 1) { $Valid } } -ReadTags {$GoodTags} -Delay {$script:Delays++} -Attempts 3
Assert ($Result.version -eq $Version -and $script:Reads -eq 2 -and $script:Delays -eq 1) '404 -> valid'
$script:Reads=0
Refuses { Wait-RegistryPublication -ReadVersion {$script:Reads++; $null} -ReadTags {$GoodTags} -Delay {} -Attempts 3 } 'accepted/pending/unverified.*VerifyOnly'
Assert ($script:Reads -eq 3) 'persistent404 bounded timeout'
$Bad=[pscustomobject]@{name='caveat-lang';version=$Version;dist=[pscustomobject]@{integrity='wrong'}}
$script:Reads=0; $script:Delays=0
Refuses { Wait-RegistryPublication -ReadVersion {$script:Reads++; $Bad} -ReadTags {$GoodTags} -Delay {$script:Delays++} } 'identity/integrity'
Assert ($script:Reads -eq 1 -and $script:Delays -eq 0) 'mismatch immediate'
Refuses { Wait-RegistryPublication -ReadVersion {throw 'HTTP503'} -ReadTags {$GoodTags} -Delay {throw 'must not delay'} } 'HTTP503'
$script:Tags=0
$Result=Wait-RegistryPublication -ReadVersion {$Valid} -ReadTags {$script:Tags++; if($script:Tags -eq 1){$OldTags}else{$GoodTags}} -Delay {} -Attempts 3
Assert ($script:Tags -eq 2 -and $Result.version -eq $Version) 'next propagation'
Refuses { Wait-RegistryPublication -ReadVersion {$Valid} -ReadTags {$OldTags} -Delay {} -Attempts 2 } 'accepted/pending/unverified'
foreach($Tags in @([pscustomobject]@{next='0.1.0-rc.8';latest='0.1.0-rc.5'},[pscustomobject]@{next=$Version;latest=$Version})) {
    Refuses { Wait-RegistryPublication -ReadVersion {$Valid} -ReadTags {$Tags} -Delay {throw 'must not delay'} } 'Unexpected dist-tags'
}
$script:Published=0; $script:Waited=0
foreach($Existing in @($null,$Valid)) {
    Invoke-PublicationDecision -Existing $Existing -VerificationOnly $true -Publish {$script:Published++} -Wait {$script:Waited++; $Valid} | Out-Null
}
Refuses { Invoke-PublicationDecision -Existing $null -VerificationOnly $true -Publish {$script:Published++} -Wait {throw 'HTTP404 timeout'} } 'HTTP404'
Assert ($script:Published -eq 0 -and $script:Waited -eq 2) 'VerifyOnly must never publish'
# Actual HTTP404/non404 wrapper behavior, without network.
$Registry='https://registry.npmjs.org'
function Invoke-RestMethod { throw [Net.WebException]::new('unavailable',$null,[Net.WebExceptionStatus]::ProtocolError,$null) }
Refuses { Get-RegistryVersion } 'unavailable'
Remove-Item Function:Invoke-RestMethod
# Actual Node parse + PowerShell5.1 deserialization of selected lock entry.
$Node=(Get-Command node -ErrorAction Stop).Source
$Temp=Join-Path ([IO.Path]::GetTempPath()) ('caveat-lock-test-'+[Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Temp | Out-Null
Push-Location $Temp
try {
    '{"packages":{"":{"name":"consumer"},"node_modules/caveat-lang":{"version":"0.1.0-rc.7","integrity":"expected","resolved":"https://registry.npmjs.org/caveat-lang/-/caveat-lang-0.1.0-rc.7.tgz"}}}' | Set-Content -LiteralPath package-lock.json -Encoding ASCII
    $Selected=Read-InstalledLock
    Assert ($Selected.version -eq $Version -and $Selected.integrity -eq $ExpectedIntegrity -and ([Uri]$Selected.resolved).Host -eq 'registry.npmjs.org') 'empty-key lock safe selection'
} finally { Pop-Location; Remove-Item -LiteralPath $Temp -Recurse -Force }
Write-Host "PASS propagation, refusal, VerifyOnly and actual lock parsing on PowerShell $($PSVersionTable.PSVersion)"
