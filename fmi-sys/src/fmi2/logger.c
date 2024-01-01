#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

#include "hdrs/fmi2FunctionTypes.h"

extern void callback_log(fmi2ComponentEnvironment componentEnvironment,
                         fmi2String instanceName, fmi2Status status,
                         fmi2String category, fmi2String message);

void callback_logger_handler(fmi2ComponentEnvironment componentEnvironment,
                             fmi2String instanceName, fmi2Status status,
                             fmi2String category, fmi2String message, ...) {
  va_list args;

  va_start(args, message);
  int buffer_size = vsnprintf(NULL, 0, message, args);
  va_end(args);
  if(buffer_size > 0) {
    // vsnprintf return value doesn't include the terminating null-byte
    char* buffer = malloc(buffer_size+1);

    if(buffer) {
      va_start(args, message);
      vsprintf(buffer, message, args);
      va_end(args);

      callback_log(componentEnvironment, instanceName, status, category, buffer);

      free(buffer);
    }
  }

}
