#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "motor_abi.h"

typedef enum Mode {
  MODE_ENABLE,
  MODE_DISABLE,
  MODE_MIT,
  MODE_POS_VEL,
  MODE_VEL,
  MODE_FORCE_POS,
  MODE_PING,
  MODE_READ_PARAM,
  MODE_WRITE_PARAM
} Mode;

typedef enum Vendor {
  VENDOR_DAMIAO,
  VENDOR_ROBSTRIDE
} Vendor;

typedef struct Options {
  const char* channel;
  Vendor vendor;
  const char* model;
  uint16_t motor_id;
  uint16_t feedback_id;
  Mode mode;
  int loop;
  int dt_ms;
  int ensure_mode;
  int ensure_timeout_ms;
  int ensure_strict;
  int print_state;
  float pos;
  float vel;
  float kp;
  float kd;
  float tau;
  float vlim;
  float ratio;
  uint16_t param_id;
  const char* param_type;
  const char* param_value;
  int param_timeout_ms;
} Options;

static void print_help(void) {
  puts("c_abi_demo (multi-mode)\n"
       "Usage:\n"
       "  ./c_abi_demo --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \\\n"
       "    --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 200 --dt-ms 20\n\n"
       "Modes:\n"
       "  Damiao: enable | disable | mit | pos-vel | vel | force-pos\n"
       "  RobStride: ping | enable | disable | mit | vel | read-param | write-param\n\n"
       "Common:\n"
       "  --vendor --channel --model --motor-id --feedback-id --loop --dt-ms\n"
       "  --ensure-mode 1/0 --ensure-timeout-ms --ensure-strict 1/0 --print-state 1/0\n"
       "Control params:\n"
       "  MIT: --pos --vel --kp --kd --tau\n"
       "  POS_VEL: --pos --vlim\n"
       "  VEL: --vel\n"
       "  FORCE_POS: --pos --vlim --ratio\n"
       "  RobStride param ops: --param-id --param-type i8|u8|u16|u32|f32 --param-value --param-timeout-ms");
}

static int parse_i(const char* s, int* out) {
  char* end = NULL;
  long v = strtol(s, &end, 0);
  if (!s || !*s || (end && *end)) return -1;
  *out = (int)v;
  return 0;
}

static int parse_u16(const char* s, uint16_t* out) {
  char* end = NULL;
  unsigned long v = strtoul(s, &end, 0);
  if (!s || !*s || (end && *end) || v > 0xFFFFUL) return -1;
  *out = (uint16_t)v;
  return 0;
}

static int parse_f(const char* s, float* out) {
  char* end = NULL;
  float v = strtof(s, &end);
  if (!s || !*s || (end && *end)) return -1;
  *out = v;
  return 0;
}

static int parse_mode(const char* s, Mode* out) {
  if (strcmp(s, "enable") == 0) *out = MODE_ENABLE;
  else if (strcmp(s, "disable") == 0) *out = MODE_DISABLE;
  else if (strcmp(s, "mit") == 0) *out = MODE_MIT;
  else if (strcmp(s, "pos-vel") == 0) *out = MODE_POS_VEL;
  else if (strcmp(s, "vel") == 0) *out = MODE_VEL;
  else if (strcmp(s, "force-pos") == 0) *out = MODE_FORCE_POS;
  else if (strcmp(s, "ping") == 0) *out = MODE_PING;
  else if (strcmp(s, "read-param") == 0) *out = MODE_READ_PARAM;
  else if (strcmp(s, "write-param") == 0) *out = MODE_WRITE_PARAM;
  else return -1;
  return 0;
}

static int parse_vendor(const char* s, Vendor* out) {
  if (strcmp(s, "damiao") == 0) *out = VENDOR_DAMIAO;
  else if (strcmp(s, "robstride") == 0) *out = VENDOR_ROBSTRIDE;
  else return -1;
  return 0;
}

static int parse_args(int argc, char** argv, Options* o) {
  for (int i = 1; i < argc; ++i) {
    const char* k = argv[i];
    if (strcmp(k, "--help") == 0) {
      print_help();
      return 1;
    }
    if (i + 1 >= argc) {
      fprintf(stderr, "missing value for %s\n", k);
      return -1;
    }
    const char* v = argv[++i];
    if (strcmp(k, "--vendor") == 0) {
      if (parse_vendor(v, &o->vendor) != 0) return -1;
    } else if (strcmp(k, "--channel") == 0) o->channel = v;
    else if (strcmp(k, "--model") == 0) o->model = v;
    else if (strcmp(k, "--motor-id") == 0) {
      if (parse_u16(v, &o->motor_id) != 0) return -1;
    } else if (strcmp(k, "--feedback-id") == 0) {
      if (parse_u16(v, &o->feedback_id) != 0) return -1;
    } else if (strcmp(k, "--mode") == 0) {
      if (parse_mode(v, &o->mode) != 0) return -1;
    } else if (strcmp(k, "--loop") == 0) {
      if (parse_i(v, &o->loop) != 0) return -1;
    } else if (strcmp(k, "--dt-ms") == 0) {
      if (parse_i(v, &o->dt_ms) != 0) return -1;
    } else if (strcmp(k, "--ensure-mode") == 0) {
      if (parse_i(v, &o->ensure_mode) != 0) return -1;
    } else if (strcmp(k, "--ensure-timeout-ms") == 0) {
      if (parse_i(v, &o->ensure_timeout_ms) != 0) return -1;
    } else if (strcmp(k, "--ensure-strict") == 0) {
      if (parse_i(v, &o->ensure_strict) != 0) return -1;
    } else if (strcmp(k, "--print-state") == 0) {
      if (parse_i(v, &o->print_state) != 0) return -1;
    } else if (strcmp(k, "--pos") == 0) {
      if (parse_f(v, &o->pos) != 0) return -1;
    } else if (strcmp(k, "--vel") == 0) {
      if (parse_f(v, &o->vel) != 0) return -1;
    } else if (strcmp(k, "--kp") == 0) {
      if (parse_f(v, &o->kp) != 0) return -1;
    } else if (strcmp(k, "--kd") == 0) {
      if (parse_f(v, &o->kd) != 0) return -1;
    } else if (strcmp(k, "--tau") == 0) {
      if (parse_f(v, &o->tau) != 0) return -1;
    } else if (strcmp(k, "--vlim") == 0) {
      if (parse_f(v, &o->vlim) != 0) return -1;
    } else if (strcmp(k, "--ratio") == 0) {
      if (parse_f(v, &o->ratio) != 0) return -1;
    } else if (strcmp(k, "--param-id") == 0) {
      if (parse_u16(v, &o->param_id) != 0) return -1;
    } else if (strcmp(k, "--param-type") == 0) {
      o->param_type = v;
    } else if (strcmp(k, "--param-value") == 0) {
      o->param_value = v;
    } else if (strcmp(k, "--param-timeout-ms") == 0) {
      if (parse_i(v, &o->param_timeout_ms) != 0) return -1;
    } else {
      fprintf(stderr, "unknown arg: %s\n", k);
      return -1;
    }
  }
  return 0;
}

static int check_rc(int32_t rc, const char* what) {
  if (rc == 0) return 0;
  fprintf(stderr, "%s failed: %s\n", what, motor_last_error_message());
  return -1;
}

static uint32_t abi_mode(Mode m) {
  switch (m) {
    case MODE_MIT: return 1;
    case MODE_POS_VEL: return 2;
    case MODE_VEL: return 3;
    case MODE_FORCE_POS: return 4;
    default: return 1;
  }
}

static const char* vendor_name(Vendor v) {
  switch (v) {
    case VENDOR_DAMIAO: return "damiao";
    case VENDOR_ROBSTRIDE: return "robstride";
    default: return "unknown";
  }
}

static const char* mode_name(Mode m) {
  switch (m) {
    case MODE_ENABLE: return "enable";
    case MODE_DISABLE: return "disable";
    case MODE_MIT: return "mit";
    case MODE_POS_VEL: return "pos-vel";
    case MODE_VEL: return "vel";
    case MODE_FORCE_POS: return "force-pos";
    case MODE_PING: return "ping";
    case MODE_READ_PARAM: return "read-param";
    case MODE_WRITE_PARAM: return "write-param";
    default: return "unknown";
  }
}

static int print_state(MotorHandle* motor, const char* prefix) {
  MotorState st = {0};
  if (check_rc(motor_handle_get_state(motor, &st), "get_state") != 0) return -1;
  if (st.has_value) {
    printf("%s pos=%+.3f vel=%+.3f torq=%+.3f status=%u arb=0x%X\n", prefix, st.pos, st.vel,
           st.torq, st.status_code, st.arbitration_id);
  } else {
    printf("%s no feedback yet\n", prefix);
  }
  return 0;
}

static int do_robstride_read(MotorHandle* motor, const Options* o) {
  if (strcmp(o->param_type, "i8") == 0) {
    int8_t value = 0;
    if (check_rc(motor_handle_robstride_get_param_i8(motor, o->param_id,
                                                     (uint32_t)o->param_timeout_ms, &value),
                 "robstride_get_param_i8") != 0)
      return -1;
    printf("param 0x%04X (%s) = %d\n", o->param_id, o->param_type, value);
    return 0;
  }
  if (strcmp(o->param_type, "u8") == 0) {
    uint8_t value = 0;
    if (check_rc(motor_handle_robstride_get_param_u8(motor, o->param_id,
                                                     (uint32_t)o->param_timeout_ms, &value),
                 "robstride_get_param_u8") != 0)
      return -1;
    printf("param 0x%04X (%s) = %u\n", o->param_id, o->param_type, value);
    return 0;
  }
  if (strcmp(o->param_type, "u16") == 0) {
    uint16_t value = 0;
    if (check_rc(motor_handle_robstride_get_param_u16(motor, o->param_id,
                                                      (uint32_t)o->param_timeout_ms, &value),
                 "robstride_get_param_u16") != 0)
      return -1;
    printf("param 0x%04X (%s) = %u\n", o->param_id, o->param_type, value);
    return 0;
  }
  if (strcmp(o->param_type, "u32") == 0) {
    uint32_t value = 0;
    if (check_rc(motor_handle_robstride_get_param_u32(motor, o->param_id,
                                                      (uint32_t)o->param_timeout_ms, &value),
                 "robstride_get_param_u32") != 0)
      return -1;
    printf("param 0x%04X (%s) = %u\n", o->param_id, o->param_type, value);
    return 0;
  }
  {
    float value = 0.0f;
    if (check_rc(motor_handle_robstride_get_param_f32(motor, o->param_id,
                                                      (uint32_t)o->param_timeout_ms, &value),
                 "robstride_get_param_f32") != 0)
      return -1;
    printf("param 0x%04X (%s) = %.6f\n", o->param_id, o->param_type, value);
    return 0;
  }
}

static int do_robstride_write(MotorHandle* motor, const Options* o) {
  if (strcmp(o->param_type, "i8") == 0) {
    int value = 0;
    if (parse_i(o->param_value, &value) != 0) return -1;
    return check_rc(motor_handle_robstride_write_param_i8(motor, o->param_id, (int8_t)value),
                    "robstride_write_param_i8");
  }
  if (strcmp(o->param_type, "u8") == 0) {
    uint16_t value = 0;
    if (parse_u16(o->param_value, &value) != 0) return -1;
    return check_rc(motor_handle_robstride_write_param_u8(motor, o->param_id, (uint8_t)value),
                    "robstride_write_param_u8");
  }
  if (strcmp(o->param_type, "u16") == 0) {
    uint16_t value = 0;
    if (parse_u16(o->param_value, &value) != 0) return -1;
    return check_rc(motor_handle_robstride_write_param_u16(motor, o->param_id, value),
                    "robstride_write_param_u16");
  }
  if (strcmp(o->param_type, "u32") == 0) {
    int value = 0;
    if (parse_i(o->param_value, &value) != 0) return -1;
    return check_rc(motor_handle_robstride_write_param_u32(motor, o->param_id, (uint32_t)value),
                    "robstride_write_param_u32");
  }
  {
    float value = 0.0f;
    if (parse_f(o->param_value, &value) != 0) return -1;
    return check_rc(motor_handle_robstride_write_param_f32(motor, o->param_id, value),
                    "robstride_write_param_f32");
  }
}

int main(int argc, char** argv) {
  Options o = {
      .channel = "can0",
      .vendor = VENDOR_DAMIAO,
      .model = "4340",
      .motor_id = 0x01,
      .feedback_id = 0x11,
      .mode = MODE_MIT,
      .loop = 100,
      .dt_ms = 20,
      .ensure_mode = 1,
      .ensure_timeout_ms = 1000,
      .ensure_strict = 0,
      .print_state = 1,
      .pos = 0.0f,
      .vel = 0.0f,
      .kp = 30.0f,
      .kd = 1.0f,
      .tau = 0.0f,
      .vlim = 1.0f,
      .ratio = 0.3f,
      .param_id = 0x7019,
      .param_type = "f32",
      .param_value = "0",
      .param_timeout_ms = 1000,
  };

  int pr = parse_args(argc, argv, &o);
  if (pr == 1) return 0;
  if (pr != 0) {
    print_help();
    return 2;
  }

  if (o.vendor == VENDOR_ROBSTRIDE) {
    if (strcmp(o.model, "4340") == 0) o.model = "rs-00";
    if (o.feedback_id == 0x11) o.feedback_id = 0xFF;
  }

  printf("vendor=%s channel=%s model=%s motor_id=0x%X feedback_id=0x%X mode=%s\n",
         vendor_name(o.vendor), o.channel, o.model, o.motor_id, o.feedback_id, mode_name(o.mode));

  MotorController* controller = motor_controller_new_socketcan(o.channel);
  if (!controller) {
    fprintf(stderr, "create controller failed: %s\n", motor_last_error_message());
    return 1;
  }

  MotorHandle* motor = o.vendor == VENDOR_DAMIAO
                           ? motor_controller_add_damiao_motor(controller, o.motor_id,
                                                               o.feedback_id, o.model)
                           : motor_controller_add_robstride_motor(controller, o.motor_id,
                                                                  o.feedback_id, o.model);
  if (!motor) {
    fprintf(stderr, "add motor failed: %s\n", motor_last_error_message());
    motor_controller_free(controller);
    return 1;
  }

  if (o.vendor == VENDOR_DAMIAO &&
      (o.mode == MODE_PING || o.mode == MODE_READ_PARAM || o.mode == MODE_WRITE_PARAM)) {
    fprintf(stderr, "Damiao demo does not support robstride-only modes\n");
    goto out;
  }
  if (o.vendor == VENDOR_ROBSTRIDE &&
      (o.mode == MODE_POS_VEL || o.mode == MODE_FORCE_POS)) {
    fprintf(stderr, "RobStride demo supports ping/enable/disable/mit/vel/read-param/write-param\n");
    goto out;
  }

  if (o.mode == MODE_PING) {
    uint8_t device_id = 0;
    uint8_t responder_id = 0;
    if (check_rc(motor_handle_robstride_ping(motor, &device_id, &responder_id), "robstride_ping") != 0)
      goto out;
    printf("ping ok device_id=%u responder_id=%u\n", device_id, responder_id);
    (void)print_state(motor, "[state]");
    goto out;
  }

  if (o.mode == MODE_READ_PARAM) {
    if (do_robstride_read(motor, &o) != 0) goto out;
    (void)print_state(motor, "[state]");
    goto out;
  }

  if (o.mode == MODE_WRITE_PARAM) {
    if (do_robstride_write(motor, &o) != 0) goto out;
    if (do_robstride_read(motor, &o) != 0) goto out;
    (void)print_state(motor, "[state]");
    goto out;
  }

  if (o.mode != MODE_ENABLE && o.mode != MODE_DISABLE) {
    if (check_rc(motor_controller_enable_all(controller), "enable_all") != 0) goto out;
    usleep(300000);
  }

  if (o.ensure_mode && o.mode != MODE_ENABLE && o.mode != MODE_DISABLE) {
    int32_t rc = motor_handle_ensure_mode(motor, abi_mode(o.mode), (uint32_t)o.ensure_timeout_ms);
    if (rc != 0) {
      if (o.ensure_strict) {
        check_rc(rc, "ensure_mode");
        goto out;
      }
      fprintf(stderr, "[warn] ensure_mode failed: %s; continue anyway\n",
              motor_last_error_message());
    }
  }

  for (int i = 0; i < o.loop; ++i) {
    int32_t rc = 0;
    switch (o.mode) {
      case MODE_ENABLE:
        rc = motor_handle_enable(motor);
        if (check_rc(rc, "enable") != 0) goto out;
        (void)motor_handle_request_feedback(motor);
        break;
      case MODE_DISABLE:
        rc = motor_handle_disable(motor);
        if (check_rc(rc, "disable") != 0) goto out;
        (void)motor_handle_request_feedback(motor);
        break;
      case MODE_MIT:
        rc = motor_handle_send_mit(motor, o.pos, o.vel, o.kp, o.kd, o.tau);
        if (check_rc(rc, "send_mit") != 0) goto out;
        break;
      case MODE_POS_VEL:
        rc = motor_handle_send_pos_vel(motor, o.pos, o.vlim);
        if (check_rc(rc, "send_pos_vel") != 0) goto out;
        break;
      case MODE_VEL:
        rc = motor_handle_send_vel(motor, o.vel);
        if (check_rc(rc, "send_vel") != 0) goto out;
        break;
      case MODE_FORCE_POS:
        rc = motor_handle_send_force_pos(motor, o.pos, o.vlim, o.ratio);
        if (check_rc(rc, "send_force_pos") != 0) goto out;
        break;
    }

    if (o.print_state) {
      char prefix[32];
      snprintf(prefix, sizeof(prefix), "#%d", i);
      if (print_state(motor, prefix) != 0) goto out;
    }
    if (o.dt_ms > 0) usleep((useconds_t)o.dt_ms * 1000);
  }

out:
  motor_controller_shutdown(controller);
  motor_handle_free(motor);
  motor_controller_free(controller);
  return 0;
}
