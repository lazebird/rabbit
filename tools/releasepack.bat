REM @echo off
if "%1"=="Debug" Goto :eof
pushd %~dp0
REM update version
cd ..\lazebird.rabbit.rabbit\
copy vgen.auto.txt ..\release\version.txt
REM update binary
cd bin\Release\
..\..\..\tools\ILMerge.exe /targetplatform:v4 /ndebug /target:winexe /out:../../../release/sRabbit.exe lazebird.rabbit.rabbit.exe /wildcards *.dll
REM add backup
cd ..\..\..\release\
set dirname=%date:~0,4%-%date:~5,2%-%date:~8,2%
md %dirname%
copy version.txt %dirname%\version.txt
copy sRabbit.exe %dirname%\sRabbit.exe
popd
REM pause
