REM @echo off
pushd %~dp0
echo %cd%
set strDate=%Date:~0,4%.%Date:~5,2%.%Date:~8,2%
echo %strDate% > ../release/version.txt
popd
REM pause
