param([string]$WxsFile)
$md5     = [System.Security.Cryptography.MD5]::Create()
$content = Get-Content $WxsFile -Raw

# Match: <Component Id="XXXX" Guid="*">  and replace Guid with MD5-based stable GUID
$content = [regex]::Replace(
    $content,
    '(<Component\s+Id=")([^"]+)("\s+Guid=")\*(")',
    {
        param($m)
        $id    = $m.Groups[2].Value
        $bytes = [System.Text.Encoding]::UTF8.GetBytes("gow-rust-1.0:$id")
        $hash  = $md5.ComputeHash($bytes)
        $hex   = [System.BitConverter]::ToString($hash) -replace '-', ''
        $guid  = "$($hex.Substring(0,8))-$($hex.Substring(8,4))-$($hex.Substring(12,4))-$($hex.Substring(16,4))-$($hex.Substring(20,12))"
        "$($m.Groups[1].Value)$id$($m.Groups[3].Value){$($guid.ToUpper())}$($m.Groups[4].Value)"
    }
)
[System.IO.File]::WriteAllText(
    (Resolve-Path $WxsFile).Path,
    $content,
    [System.Text.Encoding]::UTF8
)
$count = ([regex]::Matches($content, '<Component ')).Count
Write-Host "  Stable GUIDs written for $count components in $WxsFile"
