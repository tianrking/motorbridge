#include <chrono>
#include <iostream>
#include <string>
#include <thread>

#include "motorbridge/motorbridge.hpp"

int main(int argc, char** argv) {
  try {
    std::string channel = "can0";
    std::string model = "rs-00";
    uint16_t motor_id = 127;
    uint16_t feedback_id = 0xFF;
    std::string mode = "ping";

    if (argc > 1) channel = argv[1];
    if (argc > 2) model = argv[2];
    if (argc > 3) motor_id = static_cast<uint16_t>(std::stoul(argv[3], nullptr, 0));
    if (argc > 4) feedback_id = static_cast<uint16_t>(std::stoul(argv[4], nullptr, 0));
    if (argc > 5) mode = argv[5];

    motorbridge::Controller ctrl(channel);
    auto motor = ctrl.add_robstride_motor(motor_id, feedback_id, model);

    if (mode == "ping") {
      auto [device_id, responder_id] = motor.robstride_ping();
      std::cout << "ping ok device_id=" << static_cast<int>(device_id)
                << " responder_id=" << static_cast<int>(responder_id) << "\n";
      auto st = motor.get_state();
      if (st.has_value()) std::cout << "state pos=" << st->pos << " vel=" << st->vel << "\n";
      ctrl.shutdown();
      return 0;
    }

    if (mode == "read-param") {
      float mech_pos = motor.robstride_get_param_f32(0x7019);
      float vbus = motor.robstride_get_param_f32(0x701C);
      std::cout << "mechanical_position=" << mech_pos << " vbus=" << vbus << "\n";
      ctrl.shutdown();
      return 0;
    }

    ctrl.enable_all();
    motor.ensure_mode(mode == "mit" ? motorbridge::Mode::MIT : motorbridge::Mode::VEL, 1000);

    for (int i = 0; i < 20; ++i) {
      if (mode == "mit") {
        motor.send_mit(0.0f, 0.0f, 8.0f, 0.2f, 0.0f);
      } else {
        motor.send_vel(0.2f);
      }
      auto st = motor.get_state();
      if (st.has_value()) {
        std::cout << "#" << i << " pos=" << st->pos << " vel=" << st->vel << " torq=" << st->torq
                  << " status=" << static_cast<int>(st->status_code) << "\n";
      } else {
        std::cout << "#" << i << " no feedback yet\n";
      }
      std::this_thread::sleep_for(std::chrono::milliseconds(50));
    }

    ctrl.shutdown();
    return 0;
  } catch (const std::exception& e) {
    std::cerr << "error: " << e.what() << "\n";
    return 1;
  }
}
