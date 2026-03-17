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
      << "pos_repl_demo: interactive position control (POS_VEL)\n"
      << "Usage:\n"
      << "  ./pos_repl_demo --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11\n"
      << "                 [--vlim 1.5] [--dt-ms 20] [--ensure-timeout-ms 1000] [--print-state 1]\n"
      << "Input examples after start:\n"
      << "  1\n"
      << "  3.14\n"
      << "Commands: q / quit / exit / help\n";
}

int main(int argc, char** argv) {
  std::string channel = "can0";
  std::string model = "4310";
  uint16_t motor_id = 0x01;
  uint16_t feedback_id = 0x11;
  float vlim = 1.5f;
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
      } else if (k == "--vlim") {
        need_val("--vlim");
        vlim = std::stof(argv[++i]);
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

  try {
    motorbridge::Controller ctrl(channel);
    auto m = ctrl.add_damiao_motor(motor_id, feedback_id, model);

    ctrl.enable_all();
    std::this_thread::sleep_for(std::chrono::milliseconds(300));
    m.ensure_mode(motorbridge::Mode::POS_VEL, ensure_timeout_ms);

    std::cout << "ready. input target pos (rad), or q to quit\n> ";
    std::cout.flush();

    std::string line;
    while (std::getline(std::cin, line)) {
      if (line.empty()) {
        std::cout << "> ";
        std::cout.flush();
        continue;
      }
      if (line == "q" || line == "quit" || line == "exit") break;
      if (line == "help") {
        print_help();
        std::cout << "> ";
        std::cout.flush();
        continue;
      }
      try {
        float target = std::stof(line);
        m.send_pos_vel(target, vlim);
        if (dt_ms > 0) std::this_thread::sleep_for(std::chrono::milliseconds(dt_ms));
        if (print_state) {
          auto st = m.get_state();
          if (st.has_value()) {
            std::cout << "target=" << std::showpos << std::fixed << std::setprecision(3) << target
                      << " pos=" << st->pos << " vel=" << st->vel << " torq=" << st->torq
                      << std::noshowpos << " status=" << static_cast<int>(st->status_code) << "\n";
          } else {
            std::cout << "target=" << target << " (no feedback yet)\n";
          }
        } else {
          std::cout << "target sent: " << target << "\n";
        }
      } catch (const std::exception&) {
        std::cout << "invalid input: " << line << "\n";
      }
      std::cout << "> ";
      std::cout.flush();
    }

    std::cout << "bye\n";
    return 0;
  } catch (const std::exception& e) {
    std::cerr << "error: " << e.what() << "\n";
    return 1;
  }
}

