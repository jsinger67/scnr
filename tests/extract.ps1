<#
    Extracts x2 macros from a file and converts them to td! macros.
#>
#[CmdLetBinding()]
param(
    [string]$Path = "C:\Users\joerg\Downloads\test_utf8.c"
)

Get-Content $Path |
Where-Object { $_ -match "^\s*x2\("
} |
ForEach-Object {
    # x2("<pattern>", "<input>", <match_start>, <match_end>);
    $matched = $_ -match 'x2\("(?<pattern>.*)",\s*"(?<input_string>.*)",\s*(?<span_start>\d+),\s*(?<span_end>\d+)\s*\);'
    if ($matched) {
        Write-Host "Matched: $_"
        $pattern = $matches['pattern']
        if ($pattern -eq $null) {
            $pattern = ""
        }
        $pattern = $pattern -replace '\\\\', '\'
        $input_string = $matches['input_string']
        if ($input_string -eq $null) {
            $input_string = ""
        }
        $span_start = [int]$matches['span_start']
        $span_end = [int]$matches['span_end']
        try {
            $matched_substring = $input_string.Substring($span_start, $span_end - $span_start)
            $expected_match = "(`"$matched_substring`", $span_start, $span_end)"
            if ($expected_match -eq '("", 0, 0)') {
                # ("", 0, 0) is the value for no match
                $expected_match = ""
            }
            Write-Output "// td!(r`#`"$pattern`"`#, `"$input_string`", &[$expected_match]),"
        }
        catch {
            # Write-Error "Error: $_"
        }
    }
}