@echo off

set URL=https://github.com/Jupiee/rawst/releases/download/0.2/rawst.exe
set DESTINATION=C:\Users\%USERNAME%\AppData\Local\Microsoft\WindowsApps

curl -L "%URL%" -o "rawst.exe"

move "rawst.exe" "%DESTINATION%"