# This script is only for Windows
$3rd_party = "./3rd_party"
$path_ort = "${3rd_party}/ort"
if ( -not (Test-Path $path_ort) ) {
  $zip_onnx = "${3rd_party}/ort.zip"
  if ( -not (Test-Path $zip_onnx)) {
    Invoke-WebRequest -Uri "https://github.com/microsoft/onnxruntime/releases/download/v1.14.1/onnxruntime-win-x64-1.14.1.zip" -OutFile $zip_onnx
  }
  Expand-Archive -Path $zip_onnx -DestinationPath "$3rd_party/ort"
  Move-Item "$3rd_party/ort/onnxruntime-win-x64-1.14.1/*" $path_ort
}