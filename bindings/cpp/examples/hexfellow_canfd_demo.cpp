#include <chrono>
#include <cstdint>
#include <iostream>
#include <string>
#include <thread>

#include "motorbridge/motorbridge.hpp"

int main(int argc, char** argv) {
  std::string channel = "can0";
  std::string model = "hexfellow";
  uint16_t motor_id = 0x01;
  uint16_t feedback_id = 0x00;
  float pos = 0.8f;
  float vel = 1.0f;
  float vlim = 1.0f;
  float kp = 30.0f;
  float kd = 1.0f;
  float tau = 0.1f;
  int loop = 20;
  int dt_ms = 50;
  bool use_mit = true;

  for (int i = 1; i < argc; ++i) {
    std::string k = argv[i];
    if (i + 1 >= argc) break;
    std::string v = argv[++i];
    if (k == "--channel")
      channel = v;
    else if (k == "--model")
      model = v;
    else if (k == "--motor-id")
      motor_id = static_cast<uint16_t>(std::stoul(v, nullptr, 0));
    else if (k == "--feedback-id")
      feedback_id = static_cast<uint16_t>(std::stoul(v, nullptr, 0));
    else if (k == "--mode")
      use_mit = (v == "mit");
    else if (k == "--loop")
      loop = std::stoi(v);
    else if (k == "--dt-ms")
      dt_ms = std::stoi(v);
    else if (k == "--pos")
      pos = std::stof(v);
    else if (k == "--vel")
      vel = std::stof(v);
    else if (k == "--vlim")
      vlim = std::stof(v);
    else if (k == "--kp")
      kp = std::stof(v);
    else if (k == "--kd")
      kd = std::stof(v);
    else if (k == "--tau")
      tau = std::stof(v);
  }

  try {
    auto ctrl = motorbridge::Controller::from_socketcanfd(channel);
    auto m = ctrl.add_hexfellow_motor(motor_id, feedback_id, model);

    std::cout << "vendor=hexfellow transport=socketcanfd channel=" << channel
              << " model=" << model << " motor_id=0x" << std::hex << motor_id
              << " feedback_id=0x" << feedback_id << std::dec
              << " mode=" << (use_mit ? "mit" : "pos-vel") << "\n";

    ctrl.enable_all();
    m.ensure_mode(use_mit ? motorbridge::Mode::MIT : motorbridge::Mode::POS_VEL, 1000);

    for (int i = 0; i < loop; ++i) {
      if (use_mit) {
        m.send_mit(pos, vel, kp, kd, tau);
      } else {
        m.send_pos_vel(pos, vlim);
      }
      m.request_feedback();
      auto st = m.get_state();
      if (st.has_value()) {
        std::cout << "#" << i << " pos=" << st->pos << " vel=" << st->vel
                  << " torq=" << st->torq << " status="
                  << static_cast<int>(st->status_code) << "\n";
      } else {
        std::cout << "#" << i << " no feedback yet\n";
      }
      if (dt_ms > 0) {
        std::this_thread::sleep_for(std::chrono::milliseconds(dt_ms));
      }
    }

    ctrl.shutdown();
    return 0;
  } catch (const std::exception& e) {
    std::cerr << "error: " << e.what() << "\n";
    return 1;
  }
}
