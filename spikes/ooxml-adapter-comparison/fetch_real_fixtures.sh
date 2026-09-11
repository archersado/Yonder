#!/usr/bin/env sh
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)/fixtures/real"
mkdir -p "$root"
base="https://raw.githubusercontent.com/dotnet/Open-XML-SDK/main/test/DocumentFormat.OpenXml.Tests.Assets/assets/TestDataStorage/v2FxTestFiles"
curl -fL "$base/asSources/wordprocessing/complex1_NOR.docx" -o "$root/sample.docx"
curl -fL "$base/ForTestCase/worksheets.xlsx" -o "$root/sample.xlsx"
curl -fL "$base/ForTestCase/typical.pptx" -o "$root/sample.pptx"

cd "$root"
printf '%s  %s\n' \
  'e88a8f272e3c28baae8aa82875acf8d29e26cb8746b9436a2adf9a05bfc63fbe' sample.docx \
  '0307c8a9bbfe7e82fa2271be7629084a13df1051a366e24cb7318c2d2fcf7a9f' sample.xlsx \
  '02a3df49dd36c319d4c34cd05591b4e6e055d365a9b4e79045a0cc9fd2441fb2' sample.pptx \
  | sha256sum -c -
