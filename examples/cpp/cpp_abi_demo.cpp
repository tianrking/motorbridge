#include <chrono>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <string>
#include <thread>

extern "C" {
#include "motor_abi.h"
}

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
  std::string model = "4340";
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
  float kp = 30.0f;
  float kd = 1.0f;
  float tau = 0.0f;
  float vlim = 1.0f;
  float ratio = 0.3f;
};

static void print_help() {
  std::cout
      << "cpp_abi_demo (multi-mode)\n"
      << "Usage:\n"
      << "  ./cpp_abi_demo --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \\\n"
      << "    --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 200 --dt-ms 20\n\n"
      << "Modes:\n"
      << "  enable | disable | mit | pos-vel | vel | force-pos\n\n"
      << "Common:\n"
      << "  --channel --model --motor-id --feedback-id --loop --dt-ms\n"
      << "  --ensure-mode 1/0 --ensure-timeout-ms --ensure-strict 1/0 --print-state 1/0\n"
      << "Control params:\n"
      << "  MIT: --pos --vel --kp --kd --tau\n"
      << "  POS_VEL: --pos --vlim\n"
      << "  VEL: --vel\n"
      << "  FORCE_POS: --pos --vlim --ratio\n";
}

static bool ok(int32_t rc, const char* what) {
  if (rc == 0) return true;
  std::cerr << what << " failed: " << motor_last_error_message() << "\n";
  return false;
}

static bool parse_mode(const std::string& s, Mode& mode) {
  if (s == "enable") mode = Mode::Enable;
  else if (s == "disable") mode = Mode::Disable;
  else if (s == "mit") mode = Mode::Mit;
  else if (s == "pos-vel") mode = Mode::PosVel;
  else if (s == "vel") mode = Mode::Vel;
  else if (s == "force-pos") mode = Mode::ForcePos;
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

static uint32_t abi_mode(Mode mode) {
  switch (mode) {
    case Mode::Mit: return 1;
    case Mode::PosVel: return 2;
    case Mode::Vel: return 3;
    case Mode::ForcePos: return 4;
    default: return 1;
  }
}

static bool parse_args(int argc, char** argv, Options& o) {
  for (int i = 1; i < argc; ++i) {
    std::string k = argv[i];
    if (k == "--help") {
      print_help();
      return false;
    }
    if (i + 1 >= argc) {
      std::cerr << "missing value for " << k << "\n";
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
          std::cerr << "unknown mode: " << v << "\n";
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
        std::cerr << "unknown arg: " << k << "\n";
        return false;
      }
    } catch (...) {
      std::cerr << "invalid value for " << k << ": " << v << "\n";
      return false;
    }
  }
  return true;
}

int main(int argc, char** argv) {
  Options o;
  if (!parse_args(argc, argv, o)) {
    if (argc <= 1 || std::strcmp(argv[1], "--help") != 0) print_help();
    return argc > 1 && std::strcmp(argv[1], "--help") == 0 ? 0 : 2;
  }

  std::cout << "channel=" << o.channel << " model=" << o.model << " motor_id=0x" << std::hex
            << o.motor_id << " feedback_id=0x" << o.feedback_id << std::dec
            << " mode=" << mode_name(o.mode) << "\n";

  MotorController* controller = motor_controller_new_socketcan(o.channel.c_str());
  if (!controller) {
    std::cerr << "create controller failed: " << motor_last_error_message() << "\n";
    return 1;
  }

  MotorHandle* motor =
      motor_controller_add_damiao_motor(controller, o.motor_id, o.feedback_id, o.model.c_str());
  if (!motor) {
    std::cerr << "add motor failed: " << motor_last_error_message() << "\n";
    motor_controller_free(controller);
    return 1;
  }

  if (o.mode != Mode::Enable && o.mode != Mode::Disable) {
    if (!ok(motor_controller_enable_all(controller), "enable_all")) goto out;
    std::this_thread::sleep_for(std::chrono::milliseconds(300));
  }

  if (o.ensure_mode && o.mode != Mode::Enable && o.mode != Mode::Disable) {
    int32_t rc = motor_handle_ensure_mode(motor, abi_mode(o.mode), static_cast<uint32_t>(o.ensure_timeout_ms));
    if (rc != 0) {
      if (o.ensure_strict) {
        if (!ok(rc, "ensure_mode")) goto out;
      } else {
        std::cerr << "[warn] ensure_mode failed: " << motor_last_error_message()
                  << "; continue anyway\n";
      }
    }
  }

  for (int i = 0; i < o.loop; ++i) {
    switch (o.mode) {
      case Mode::Enable:
        if (!ok(motor_handle_enable(motor), "enable")) goto out;
        (void)motor_handle_request_feedback(motor);
        break;
      case Mode::Disable:
        if (!ok(motor_handle_disable(motor), "disable")) goto out;
        (void)motor_handle_request_feedback(motor);
        break;
      case Mode::Mit:
        if (!ok(motor_handle_send_mit(motor, o.pos, o.vel, o.kp, o.kd, o.tau), "send_mit")) goto out;
        break;
      case Mode::PosVel:
        if (!ok(motor_handle_send_pos_vel(motor, o.pos, o.vlim), "send_pos_vel")) goto out;
        break;
      case Mode::Vel:
        if (!ok(motor_handle_send_vel(motor, o.vel), "send_vel")) goto out;
        break;
      case Mode::ForcePos:
        if (!ok(motor_handle_send_force_pos(motor, o.pos, o.vlim, o.ratio), "send_force_pos")) goto out;
        break;
    }

    if (o.print_state) {
      MotorState st{};
      if (!ok(motor_handle_get_state(motor, &st), "get_state")) goto out;

      if (st.has_value) {
        std::cout << "#" << i << " pos=" << st.pos << " vel=" << st.vel << " torq=" << st.torq
                  << " status=" << static_cast<int>(st.status_code) << "\n";
      } else {
        std::cout << "#" << i << " no feedback yet\n";
      }
    }

    if (o.dt_ms > 0) std::this_thread::sleep_for(std::chrono::milliseconds(o.dt_ms));
  }

out:
  motor_controller_shutdown(controller);
  motor_handle_free(motor);
  motor_controller_free(controller);
  return 0;
}
