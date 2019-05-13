#include <stdarg.h>
#include <stdio.h>

#include "FMI2/fmi2FunctionTypes.h"

extern void callback_log(fmi2ComponentEnvironment componentEnvironment,
                         fmi2String instanceName, fmi2Status status,
                         fmi2String category, fmi2String message);

void callback_logger_handler(fmi2ComponentEnvironment componentEnvironment,
                             fmi2String instanceName, fmi2Status status,
                             fmi2String category, fmi2String message, ...) {
  char buffer[256];
  va_list args;
  va_start(args, message);
  vsprintf(buffer, message, args);
  callback_log(componentEnvironment, instanceName, status, category, buffer);
  va_end(args);
}