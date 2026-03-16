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
  MODE_FORCE_POS
} Mode;

typedef struct Options {
  const char* channel;
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
} Options;

static void print_help(void) {
  puts("c_abi_demo (multi-mode)\n"
       "Usage:\n"
       "  ./c_abi_demo --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \\\n"
       "    --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 200 --dt-ms 20\n\n"
       "Modes:\n"
       "  enable | disable | mit | pos-vel | vel | force-pos\n\n"
       "Common:\n"
       "  --channel --model --motor-id --feedback-id --loop --dt-ms\n"
       "  --ensure-mode 1/0 --ensure-timeout-ms --ensure-strict 1/0 --print-state 1/0\n"
       "Control params:\n"
       "  MIT: --pos --vel --kp --kd --tau\n"
       "  POS_VEL: --pos --vlim\n"
       "  VEL: --vel\n"
       "  FORCE_POS: --pos --vlim --ratio");
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
    if (strcmp(k, "--channel") == 0) o->channel = v;
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

static const char* mode_name(Mode m) {
  switch (m) {
    case MODE_ENABLE: return "enable";
    case MODE_DISABLE: return "disable";
    case MODE_MIT: return "mit";
    case MODE_POS_VEL: return "pos-vel";
    case MODE_VEL: return "vel";
    case MODE_FORCE_POS: return "force-pos";
    default: return "unknown";
  }
}

int main(int argc, char** argv) {
  Options o = {
      .channel = "can0",
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
  };

  int pr = parse_args(argc, argv, &o);
  if (pr == 1) return 0;
  if (pr != 0) {
    print_help();
    return 2;
  }

  printf("channel=%s model=%s motor_id=0x%X feedback_id=0x%X mode=%s\n", o.channel, o.model,
         o.motor_id, o.feedback_id, mode_name(o.mode));

  MotorController* controller = motor_controller_new_socketcan(o.channel);
  if (!controller) {
    fprintf(stderr, "create controller failed: %s\n", motor_last_error_message());
    return 1;
  }

  MotorHandle* motor =
      motor_controller_add_damiao_motor(controller, o.motor_id, o.feedback_id, o.model);
  if (!motor) {
    fprintf(stderr, "add motor failed: %s\n", motor_last_error_message());
    motor_controller_free(controller);
    return 1;
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
      MotorState st = {0};
      if (check_rc(motor_handle_get_state(motor, &st), "get_state") != 0) goto out;
      if (st.has_value) {
        printf("#%d pos=%+.3f vel=%+.3f torq=%+.3f status=%u\n", i, st.pos, st.vel, st.torq,
               st.status_code);
      } else {
        printf("#%d no feedback yet\n", i);
      }
    }
    if (o.dt_ms > 0) usleep((useconds_t)o.dt_ms * 1000);
  }

out:
  motor_controller_shutdown(controller);
  motor_handle_free(motor);
  motor_controller_free(controller);
  return 0;
}
