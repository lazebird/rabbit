@echo off
pushd %~dp0
echo %cd%
set pwd=%cd%
set /p v=Version: 
echo pwd %pwd% version %v%
call:prjpack lazebird.rabbit.ping
call:prjpack lazebird.rabbit.http
call:prjpack lazebird.rabbit.common
call:prjpack lazebird.rabbit.tftp
call:prjpack lazebird.rabbit.fs
call:prjpack lazebird.rabbit.queue
popd
REM pause
GOTO:EOF

:prjpack
REM echo arg1:%~1 pwd:%pwd% ver: %v%
cd ..\%~1
nuget.exe pack -Version %v% %~1.csproj
cd %pwd%
GOTO:EOF
