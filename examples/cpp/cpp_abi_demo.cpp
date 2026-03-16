#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <thread>

extern "C" {
#include "motor_abi.h"
}

static bool ok(int32_t rc, const char* what) {
  if (rc == 0) return true;
  std::cerr << what << " failed: " << motor_last_error_message() << "\n";
  return false;
}

int main(int argc, char** argv) {
  const char* channel = argc > 1 ? argv[1] : "can0";
  const char* model = argc > 2 ? argv[2] : "4340";
  uint16_t motor_id = argc > 3 ? static_cast<uint16_t>(std::strtoul(argv[3], nullptr, 0)) : 0x01;
  uint16_t feedback_id = argc > 4 ? static_cast<uint16_t>(std::strtoul(argv[4], nullptr, 0)) : 0x11;

  std::cout << "channel=" << channel << " model=" << model
            << " motor_id=0x" << std::hex << motor_id
            << " feedback_id=0x" << feedback_id << std::dec << "\n";

  MotorController* controller = motor_controller_new_socketcan(channel);
  if (!controller) {
    std::cerr << "create controller failed: " << motor_last_error_message() << "\n";
    return 1;
  }

  MotorHandle* motor = motor_controller_add_damiao_motor(controller, motor_id, feedback_id, model);
  if (!motor) {
    std::cerr << "add motor failed: " << motor_last_error_message() << "\n";
    motor_controller_free(controller);
    return 1;
  }

  if (!ok(motor_controller_enable_all(controller), "enable_all")) goto out;
  std::this_thread::sleep_for(std::chrono::milliseconds(500));

  if (!ok(motor_handle_ensure_mode(motor, 1, 1000), "ensure_mode(MIT)")) goto out;

  for (int i = 0; i < 200; ++i) {
    if (!ok(motor_handle_send_mit(motor, 0.0f, 0.0f, 30.0f, 1.0f, 0.0f), "send_mit")) goto out;

    MotorState st{};
    if (!ok(motor_handle_get_state(motor, &st), "get_state")) goto out;

    if (st.has_value) {
      std::cout << "pos=" << st.pos << " vel=" << st.vel << " torq=" << st.torq
                << " status=" << static_cast<int>(st.status_code) << "\n";
    }

    std::this_thread::sleep_for(std::chrono::milliseconds(20));
  }

out:
  motor_controller_shutdown(controller);
  motor_handle_free(motor);
  motor_controller_free(controller);
  return 0;
}
