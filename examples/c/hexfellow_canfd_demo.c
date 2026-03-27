#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "motor_abi.h"

static int parse_u16(const char* s, uint16_t* out) {
  char* end = NULL;
  unsigned long v = strtoul(s, &end, 0);
  if (!s || !*s || (end && *end) || v > 0xFFFFUL) return -1;
  *out = (uint16_t)v;
  return 0;
}

static int parse_i(const char* s, int* out) {
  char* end = NULL;
  long v = strtol(s, &end, 0);
  if (!s || !*s || (end && *end)) return -1;
  *out = (int)v;
  return 0;
}

static int parse_f(const char* s, float* out) {
  char* end = NULL;
  float v = strtof(s, &end);
  if (!s || !*s || (end && *end)) return -1;
  *out = v;
  return 0;
}

static int check_rc(int32_t rc, const char* what) {
  if (rc == 0) return 0;
  fprintf(stderr, "%s failed: %s\n", what, motor_last_error_message());
  return -1;
}

int main(int argc, char** argv) {
  const char* channel = "can0";
  const char* model = "hexfellow";
  uint16_t motor_id = 0x01;
  uint16_t feedback_id = 0x00;
  int loop = 20;
  int dt_ms = 50;
  int use_mit = 1; /* 1=mit, 0=pos-vel */
  float pos = 0.8f;
  float vel = 1.0f;
  float vlim = 1.0f;
  float kp = 30.0f;
  float kd = 1.0f;
  float tau = 0.1f;

  for (int i = 1; i < argc; ++i) {
    const char* k = argv[i];
    if (strcmp(k, "--help") == 0) {
      puts("hexfellow_canfd_demo\n"
           "Usage:\n"
           "  ./hexfellow_canfd_demo --channel can0 --motor-id 0x01 --feedback-id 0x00 "
           "--mode mit --loop 20 --dt-ms 50\n"
           "Notes:\n"
           "  - Hexfellow must use CAN-FD transport (this demo always uses motor_controller_new_socketcanfd).\n"
           "  - Supported modes: mit | pos-vel");
      return 0;
    }
    if (i + 1 >= argc) break;
    const char* v = argv[++i];
    if (strcmp(k, "--channel") == 0)
      channel = v;
    else if (strcmp(k, "--model") == 0)
      model = v;
    else if (strcmp(k, "--motor-id") == 0) {
      if (parse_u16(v, &motor_id) != 0) return 2;
    } else if (strcmp(k, "--feedback-id") == 0) {
      if (parse_u16(v, &feedback_id) != 0) return 2;
    } else if (strcmp(k, "--mode") == 0) {
      if (strcmp(v, "mit") == 0)
        use_mit = 1;
      else if (strcmp(v, "pos-vel") == 0)
        use_mit = 0;
      else
        return 2;
    } else if (strcmp(k, "--loop") == 0) {
      if (parse_i(v, &loop) != 0) return 2;
    } else if (strcmp(k, "--dt-ms") == 0) {
      if (parse_i(v, &dt_ms) != 0) return 2;
    } else if (strcmp(k, "--pos") == 0) {
      if (parse_f(v, &pos) != 0) return 2;
    } else if (strcmp(k, "--vel") == 0) {
      if (parse_f(v, &vel) != 0) return 2;
    } else if (strcmp(k, "--vlim") == 0) {
      if (parse_f(v, &vlim) != 0) return 2;
    } else if (strcmp(k, "--kp") == 0) {
      if (parse_f(v, &kp) != 0) return 2;
    } else if (strcmp(k, "--kd") == 0) {
      if (parse_f(v, &kd) != 0) return 2;
    } else if (strcmp(k, "--tau") == 0) {
      if (parse_f(v, &tau) != 0) return 2;
    } else {
      fprintf(stderr, "unknown arg: %s\n", k);
      return 2;
    }
  }

  printf("vendor=hexfellow transport=socketcanfd channel=%s model=%s motor_id=0x%X feedback_id=0x%X mode=%s\n",
         channel, model, motor_id, feedback_id, use_mit ? "mit" : "pos-vel");

  MotorController* controller = motor_controller_new_socketcanfd(channel);
  if (!controller) {
    fprintf(stderr, "create controller failed: %s\n", motor_last_error_message());
    return 1;
  }

  MotorHandle* motor = motor_controller_add_hexfellow_motor(controller, motor_id, feedback_id, model);
  if (!motor) {
    fprintf(stderr, "add hexfellow motor failed: %s\n", motor_last_error_message());
    motor_controller_free(controller);
    return 1;
  }

  if (check_rc(motor_controller_enable_all(controller), "enable_all") != 0) goto out;
  if (check_rc(motor_handle_ensure_mode(motor, use_mit ? 1u : 2u, 1000), "ensure_mode") != 0) goto out;

  for (int i = 0; i < loop; ++i) {
    int rc = use_mit
                 ? motor_handle_send_mit(motor, pos, vel, kp, kd, tau)
                 : motor_handle_send_pos_vel(motor, pos, vlim);
    if (check_rc(rc, use_mit ? "send_mit" : "send_pos_vel") != 0) goto out;
    (void)motor_handle_request_feedback(motor);

    MotorState st = {0};
    if (check_rc(motor_handle_get_state(motor, &st), "get_state") != 0) goto out;
    if (st.has_value) {
      printf("#%d pos=%+.3f vel=%+.3f torq=%+.3f status=%u\n", i, st.pos, st.vel, st.torq,
             st.status_code);
    } else {
      printf("#%d no feedback yet\n", i);
    }
    if (dt_ms > 0) usleep((useconds_t)dt_ms * 1000);
  }

out:
  (void)motor_controller_shutdown(controller);
  motor_handle_free(motor);
  motor_controller_free(controller);
  return 0;
}
