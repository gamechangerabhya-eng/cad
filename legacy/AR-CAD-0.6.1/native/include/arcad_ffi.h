#ifndef ARCAD_FFI_H
#define ARCAD_FFI_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct ArcadHandle ArcadHandle;
typedef struct ArcadBuffer ArcadBuffer;
const char *arcad_core_version(void);
ArcadHandle *arcad_document_new(int32_t metric);
ArcadHandle *arcad_document_open(const uint8_t *bytes, size_t length, const char *filename);
void arcad_document_free(ArcadHandle *handle);
size_t arcad_document_entity_count(const ArcadHandle *handle);
char *arcad_document_read_notes(const ArcadHandle *handle);
char *arcad_command_execute(ArcadHandle *handle, const char *command, const char *params_json);
char *arcad_command_start(ArcadHandle *handle, const char *command);
char *arcad_command_input(ArcadHandle *handle, const char *input_json);
char *arcad_document_set_viewport(ArcadHandle *handle, double width_px, double height_px);
char *arcad_document_pointer_move(ArcadHandle *handle, double x, double y, int32_t shift, double now_seconds);
char *arcad_document_pointer_click(ArcadHandle *handle, double x, double y, int32_t shift, double now_seconds);
char *arcad_document_undo(ArcadHandle *handle);
char *arcad_document_redo(ArcadHandle *handle);
ArcadBuffer *arcad_document_save(const ArcadHandle *handle, const char *filename);
ArcadBuffer *arcad_document_render_png(const ArcadHandle *handle, uint32_t width, uint32_t height);
const uint8_t *arcad_buffer_data(const ArcadBuffer *buffer);
size_t arcad_buffer_len(const ArcadBuffer *buffer);
void arcad_buffer_free(ArcadBuffer *buffer);
char *arcad_last_error(void);
void arcad_string_free(char *value);
#ifdef __cplusplus
}
#endif
#endif
