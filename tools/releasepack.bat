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
for /f "tokens=1" %%i in ('wmic os get localdatetime /value ^| find /i "LocalDateTime"') do set today=%%i  
set dirname=%today:~14,4%-%today:~19,2%-%today:~22,2%
md %dirname%
copy version.txt %dirname%\version.txt
copy sRabbit.exe %dirname%\sRabbit.exe
popd
REM pause
