#include <chrono>
#include <cstdint>
#include <cstdlib>
#include <exception>
#include <iostream>
#include <string>
#include <thread>

#include "motorbridge/motorbridge.hpp"

enum class Mode {
  Enable,
  Disable,
  Mit,
  PosVel,
  Vel,
  ForcePos,
};

struct Options {
  std::string channel = "can0";
  std::string model = "4340P";
  uint16_t motor_id = 0x01;
  uint16_t feedback_id = 0x11;
  Mode mode = Mode::Mit;
  int loop = 100;
  int dt_ms = 20;
  int ensure_mode = 1;
  int ensure_timeout_ms = 1000;
  int ensure_strict = 0;
  int print_state = 1;
  float pos = 0.0f;
  float vel = 0.0f;
  float kp = 20.0f;
  float kd = 1.0f;
  float tau = 0.0f;
  float vlim = 1.5f;
  float ratio = 0.3f;
};

static void print_help() {
  std::cout
      << "full_modes_demo (C++ wrapper, multi-mode)\\n"
      << "Usage:\\n"
      << "  ./full_modes_demo --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \\\\n"
      << "    --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 200 --dt-ms 20\\n\\n"
      << "Modes:\\n"
      << "  enable | disable | mit | pos-vel | vel | force-pos\\n\\n"
      << "Common:\\n"
      << "  --channel --model --motor-id --feedback-id --loop --dt-ms\\n"
      << "  --ensure-mode 1/0 --ensure-timeout-ms --ensure-strict 1/0 --print-state 1/0\\n"
      << "Control params:\\n"
      << "  MIT: --pos --vel --kp --kd --tau\\n"
      << "  POS_VEL: --pos --vlim\\n"
      << "  VEL: --vel\\n"
      << "  FORCE_POS: --pos --vlim --ratio\\n";
}

static bool parse_mode(const std::string& s, Mode& out) {
  if (s == "enable") out = Mode::Enable;
  else if (s == "disable") out = Mode::Disable;
  else if (s == "mit") out = Mode::Mit;
  else if (s == "pos-vel") out = Mode::PosVel;
  else if (s == "vel") out = Mode::Vel;
  else if (s == "force-pos") out = Mode::ForcePos;
  else return false;
  return true;
}

static const char* mode_name(Mode mode) {
  switch (mode) {
    case Mode::Enable: return "enable";
    case Mode::Disable: return "disable";
    case Mode::Mit: return "mit";
    case Mode::PosVel: return "pos-vel";
    case Mode::Vel: return "vel";
    case Mode::ForcePos: return "force-pos";
  }
  return "unknown";
}

static bool parse_args(int argc, char** argv, Options& o) {
  for (int i = 1; i < argc; ++i) {
    std::string k = argv[i];
    if (k == "--help") {
      print_help();
      return false;
    }
    if (i + 1 >= argc) {
      std::cerr << "missing value for " << k << "\\n";
      return false;
    }
    std::string v = argv[++i];
    try {
      if (k == "--channel") o.channel = v;
      else if (k == "--model") o.model = v;
      else if (k == "--motor-id") o.motor_id = static_cast<uint16_t>(std::stoul(v, nullptr, 0));
      else if (k == "--feedback-id") o.feedback_id = static_cast<uint16_t>(std::stoul(v, nullptr, 0));
      else if (k == "--mode") {
        if (!parse_mode(v, o.mode)) {
          std::cerr << "unknown mode: " << v << "\\n";
          return false;
        }
      } else if (k == "--loop") o.loop = std::stoi(v);
      else if (k == "--dt-ms") o.dt_ms = std::stoi(v);
      else if (k == "--ensure-mode") o.ensure_mode = std::stoi(v);
      else if (k == "--ensure-timeout-ms") o.ensure_timeout_ms = std::stoi(v);
      else if (k == "--ensure-strict") o.ensure_strict = std::stoi(v);
      else if (k == "--print-state") o.print_state = std::stoi(v);
      else if (k == "--pos") o.pos = std::stof(v);
      else if (k == "--vel") o.vel = std::stof(v);
      else if (k == "--kp") o.kp = std::stof(v);
      else if (k == "--kd") o.kd = std::stof(v);
      else if (k == "--tau") o.tau = std::stof(v);
      else if (k == "--vlim") o.vlim = std::stof(v);
      else if (k == "--ratio") o.ratio = std::stof(v);
      else {
        std::cerr << "unknown arg: " << k << "\\n";
        return false;
      }
    } catch (...) {
      std::cerr << "invalid value for " << k << "\\n";
      return false;
    }
  }
  return true;
}

int main(int argc, char** argv) {
  Options o;
  if (!parse_args(argc, argv, o)) {
    if (argc <= 1 || std::string(argv[1]) != "--help") print_help();
    return (argc > 1 && std::string(argv[1]) == "--help") ? 0 : 2;
  }

  try {
    motorbridge::Controller ctrl(o.channel);
    auto m = ctrl.add_damiao_motor(o.motor_id, o.feedback_id, o.model);

    std::cout << "channel=" << o.channel << " model=" << o.model << " motor_id=0x" << std::hex
              << o.motor_id << " feedback_id=0x" << o.feedback_id << std::dec
              << " mode=" << mode_name(o.mode) << "\\n";

    if (o.mode != Mode::Enable && o.mode != Mode::Disable) {
      ctrl.enable_all();
      std::this_thread::sleep_for(std::chrono::milliseconds(300));
    }

    if (o.ensure_mode && o.mode != Mode::Enable && o.mode != Mode::Disable) {
      try {
        motorbridge::Mode need = motorbridge::Mode::MIT;
        if (o.mode == Mode::PosVel) need = motorbridge::Mode::POS_VEL;
        if (o.mode == Mode::Vel) need = motorbridge::Mode::VEL;
        if (o.mode == Mode::ForcePos) need = motorbridge::Mode::FORCE_POS;
        m.ensure_mode(need, static_cast<uint32_t>(o.ensure_timeout_ms));
      } catch (const std::exception& e) {
        if (o.ensure_strict) throw;
        std::cerr << "[warn] ensure_mode failed: " << e.what() << "; continue anyway\\n";
      }
    }

    for (int i = 0; i < o.loop; ++i) {
      switch (o.mode) {
        case Mode::Enable:
          m.enable();
          m.request_feedback();
          break;
        case Mode::Disable:
          m.disable();
          m.request_feedback();
          break;
        case Mode::Mit:
          m.send_mit(o.pos, o.vel, o.kp, o.kd, o.tau);
          break;
        case Mode::PosVel:
          m.send_pos_vel(o.pos, o.vlim);
          break;
        case Mode::Vel:
          m.send_vel(o.vel);
          break;
        case Mode::ForcePos:
          m.send_force_pos(o.pos, o.vlim, o.ratio);
          break;
      }

      if (o.print_state) {
        auto st = m.get_state();
        if (st.has_value()) {
          std::cout << "#" << i << " pos=" << st->pos << " vel=" << st->vel << " torq=" << st->torq
                    << " status=" << static_cast<int>(st->status_code) << "\\n";
        } else {
          std::cout << "#" << i << " no feedback yet\\n";
        }
      }

      if (o.dt_ms > 0) std::this_thread::sleep_for(std::chrono::milliseconds(o.dt_ms));
    }

    return 0;
  } catch (const std::exception& e) {
    std::cerr << "error: " << e.what() << "\\n";
    return 1;
  }
}
