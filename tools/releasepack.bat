REM @echo off
if "%1"=="Debug" Goto :eof
pushd %~dp0
REM update version
echo %date:~0,10% > ../release/version.txt
REM update binary
cd ../lazebird.rabbit.rabbit/bin/Release/
..\..\..\tools\ILMerge.exe /targetplatform:v4 /ndebug /target:winexe /out:../../../release/sRabbit.exe lazebird.rabbit.rabbit.exe /wildcards *.dll zh-CN/lazebird.rabbit.rabbit.resources.dll
popd
REM pause
