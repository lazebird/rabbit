REM @echo off
if "%1"=="Debug" Goto :eof
pushd %~dp0
REM update version
cd ..\lazebird.rabbit.rabbit\
copy vgen.auto.txt ..\release\version.txt
REM update binary
cd bin\Release\
..\..\..\tools\ILMerge.exe /targetplatform:v4 /ndebug /target:winexe /out:../../../release/sRabbit.exe lazebird.rabbit.rabbit.exe /wildcards *.dll
popd
REM pause
