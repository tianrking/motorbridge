#include <chrono>
#include <cstdint>
#include <cstdlib>
#include <exception>
#include <iomanip>
#include <iostream>
#include <string>
#include <thread>

#include "motorbridge/motorbridge.hpp"

static uint16_t parse_u16(const std::string& s) {
  return static_cast<uint16_t>(std::stoul(s, nullptr, 0));
}

static void print_help() {
  std::cout
      << "pos_ctrl_demo: move one motor to target position using POS_VEL mode\n"
      << "Usage:\n"
      << "  ./pos_ctrl_demo --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \\\n"
      << "    --target-pos 3.14 --vlim 1.5 --loop 300 --dt-ms 20\n";
}

int main(int argc, char** argv) {
  std::string channel = "can0";
  std::string model = "4310";
  uint16_t motor_id = 0x01;
  uint16_t feedback_id = 0x11;
  float target_pos = 0.0f;
  bool has_target = false;
  float vlim = 1.5f;
  int loop = 300;
  int dt_ms = 20;
  uint32_t ensure_timeout_ms = 1000;
  int print_state = 1;

  for (int i = 1; i < argc; ++i) {
    std::string k = argv[i];
    auto need_val = [&](const char* name) {
      if (i + 1 >= argc) {
        std::cerr << "missing value for " << name << "\n";
        std::exit(2);
      }
    };
    try {
      if (k == "--help") {
        print_help();
        return 0;
      } else if (k == "--channel") {
        need_val("--channel");
        channel = argv[++i];
      } else if (k == "--model") {
        need_val("--model");
        model = argv[++i];
      } else if (k == "--motor-id") {
        need_val("--motor-id");
        motor_id = parse_u16(argv[++i]);
      } else if (k == "--feedback-id") {
        need_val("--feedback-id");
        feedback_id = parse_u16(argv[++i]);
      } else if (k == "--target-pos") {
        need_val("--target-pos");
        target_pos = std::stof(argv[++i]);
        has_target = true;
      } else if (k == "--vlim") {
        need_val("--vlim");
        vlim = std::stof(argv[++i]);
      } else if (k == "--loop") {
        need_val("--loop");
        loop = std::stoi(argv[++i]);
      } else if (k == "--dt-ms") {
        need_val("--dt-ms");
        dt_ms = std::stoi(argv[++i]);
      } else if (k == "--ensure-timeout-ms") {
        need_val("--ensure-timeout-ms");
        ensure_timeout_ms = static_cast<uint32_t>(std::stoul(argv[++i]));
      } else if (k == "--print-state") {
        need_val("--print-state");
        print_state = std::stoi(argv[++i]);
      } else {
        std::cerr << "unknown arg: " << k << "\n";
        return 2;
      }
    } catch (const std::exception&) {
      std::cerr << "invalid value for " << k << "\n";
      return 2;
    }
  }

  if (!has_target) {
    std::cerr << "--target-pos is required\n";
    print_help();
    return 2;
  }

  try {
    motorbridge::Controller ctrl(channel);
    auto m = ctrl.add_damiao_motor(motor_id, feedback_id, model);

    ctrl.enable_all();
    std::this_thread::sleep_for(std::chrono::milliseconds(300));
    m.ensure_mode(motorbridge::Mode::POS_VEL, ensure_timeout_ms);

    for (int i = 0; i < loop; ++i) {
      m.send_pos_vel(target_pos, vlim);
      if (print_state) {
        auto st = m.get_state();
        if (st.has_value()) {
          std::cout << "#" << i << " pos=" << std::showpos << std::fixed << std::setprecision(3)
                    << st->pos << " vel=" << st->vel << " torq=" << st->torq << std::noshowpos
                    << " status=" << static_cast<int>(st->status_code) << "\n";
        } else {
          std::cout << "#" << i << " no feedback yet\n";
        }
      }
      if (dt_ms > 0) std::this_thread::sleep_for(std::chrono::milliseconds(dt_ms));
    }
    return 0;
  } catch (const std::exception& e) {
    std::cerr << "error: " << e.what() << "\n";
    return 1;
  }
}

