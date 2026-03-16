#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "motor_abi.h"

static int check_rc(int32_t rc, const char* what) {
  if (rc == 0) return 0;
  fprintf(stderr, "%s failed: %s\n", what, motor_last_error_message());
  return -1;
}

int main(int argc, char** argv) {
  const char* channel = "can0";
  const char* model = "4340";
  uint16_t motor_id = 0x01;
  uint16_t feedback_id = 0x11;

  if (argc >= 2) channel = argv[1];
  if (argc >= 3) model = argv[2];
  if (argc >= 4) motor_id = (uint16_t)strtoul(argv[3], NULL, 0);
  if (argc >= 5) feedback_id = (uint16_t)strtoul(argv[4], NULL, 0);

  printf("channel=%s model=%s motor_id=0x%X feedback_id=0x%X\n", channel, model, motor_id,
         feedback_id);

  MotorController* controller = motor_controller_new_socketcan(channel);
  if (!controller) {
    fprintf(stderr, "create controller failed: %s\n", motor_last_error_message());
    return 1;
  }

  MotorHandle* motor =
      motor_controller_add_damiao_motor(controller, motor_id, feedback_id, model);
  if (!motor) {
    fprintf(stderr, "add motor failed: %s\n", motor_last_error_message());
    motor_controller_free(controller);
    return 1;
  }

  if (check_rc(motor_controller_enable_all(controller), "enable_all") != 0) goto out;
  usleep(500000);

  if (check_rc(motor_handle_ensure_mode(motor, 1, 1000), "ensure_mode(MIT)") != 0) goto out;

  for (int i = 0; i < 200; i++) {
    if (check_rc(motor_handle_send_mit(motor, 0.0f, 0.0f, 30.0f, 1.0f, 0.0f), "send_mit") != 0) {
      goto out;
    }

    MotorState st = {0};
    if (check_rc(motor_handle_get_state(motor, &st), "get_state") != 0) goto out;

    if (st.has_value) {
      printf("pos=%+.3f vel=%+.3f torq=%+.3f status=%u\n", st.pos, st.vel, st.torq,
             st.status_code);
    }

    usleep(20000);
  }

out:
  motor_controller_shutdown(controller);
  motor_handle_free(motor);
  motor_controller_free(controller);
  return 0;
}
