# Define input and output parameters
$inputFile = "G:\DataBackup\Games\Steam\steamapps\common\Leaf's Odyssey\leafsodyssey.exe" # Replace with the path to your binary file
$outputDirectory = "G:\loext\" # Replace with your desired output directory

# Ensure the output directory exists
if (-not (Test-Path -Path $outputDirectory)) {
    New-Item -ItemType Directory -Path $outputDirectory | Out-Null
}

# Read the binary file into a byte array
$bytes = [System.IO.File]::ReadAllBytes($inputFile)

# Convert "metStilemap_edit" to bytes
$searchBytes = [System.Text.Encoding]::ASCII.GetBytes("metStilemap_edit")

# Find all occurrences of "metStilemap_edit"
$offsets = @()
for ($i = 0; $i -le $bytes.Length - $searchBytes.Length; $i++) {
    $found = $true
    for ($j = 0; $j -lt $searchBytes.Length; $j++) {
        if ($bytes[$i + $j] -ne $searchBytes[$j]) {
            $found = $false
            break
        }
    }
    if ($found) {
        $offsets += $i
    }
}

# Process each occurrence
foreach ($offset in $offsets) {
    $dataStart = $offset + $searchBytes.Length + 36
    if ($dataStart + 500 -le $bytes.Length) {
        $data = $bytes[$dataStart..($dataStart + 499)]
        $outputFile = Join-Path -Path $outputDirectory -ChildPath ("out-{0}.txt" -f $offset)
        [System.IO.File]::WriteAllBytes($outputFile, $data)
        Write-Host "Extracted data saved to $outputFile"
    } else {
        Write-Host "Not enough data to extract 500 bytes after offset $offset"
    }
}
