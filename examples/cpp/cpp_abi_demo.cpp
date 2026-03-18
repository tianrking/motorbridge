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
  Ping,
  ReadParam,
  WriteParam,
};

enum class Vendor { Damiao, Robstride };

struct Options {
  std::string channel = "can0";
  Vendor vendor = Vendor::Damiao;
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
  uint16_t param_id = 0x7019;
  std::string param_type = "f32";
  std::string param_value = "0";
  int param_timeout_ms = 1000;
};

static void print_help() {
  std::cout
      << "cpp_abi_demo (multi-mode)\n"
      << "Usage:\n"
      << "  ./cpp_abi_demo --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \\\n"
      << "    --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 200 --dt-ms 20\n\n"
      << "Modes:\n"
      << "  Damiao: enable | disable | mit | pos-vel | vel | force-pos\n"
      << "  RobStride: ping | enable | disable | mit | vel | read-param | write-param\n\n"
      << "Common:\n"
      << "  --vendor --channel --model --motor-id --feedback-id --loop --dt-ms\n"
      << "  --ensure-mode 1/0 --ensure-timeout-ms --ensure-strict 1/0 --print-state 1/0\n"
      << "Control params:\n"
      << "  MIT: --pos --vel --kp --kd --tau\n"
      << "  POS_VEL: --pos --vlim\n"
      << "  VEL: --vel\n"
      << "  FORCE_POS: --pos --vlim --ratio\n"
      << "  RobStride param ops: --param-id --param-type i8|u8|u16|u32|f32 --param-value --param-timeout-ms\n";
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
  else if (s == "ping") mode = Mode::Ping;
  else if (s == "read-param") mode = Mode::ReadParam;
  else if (s == "write-param") mode = Mode::WriteParam;
  else return false;
  return true;
}

static bool parse_vendor(const std::string& s, Vendor& vendor) {
  if (s == "damiao") vendor = Vendor::Damiao;
  else if (s == "robstride") vendor = Vendor::Robstride;
  else return false;
  return true;
}

static const char* vendor_name(Vendor vendor) {
  switch (vendor) {
    case Vendor::Damiao: return "damiao";
    case Vendor::Robstride: return "robstride";
  }
  return "unknown";
}

static const char* mode_name(Mode mode) {
  switch (mode) {
    case Mode::Enable: return "enable";
    case Mode::Disable: return "disable";
    case Mode::Mit: return "mit";
    case Mode::PosVel: return "pos-vel";
    case Mode::Vel: return "vel";
    case Mode::ForcePos: return "force-pos";
    case Mode::Ping: return "ping";
    case Mode::ReadParam: return "read-param";
    case Mode::WriteParam: return "write-param";
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
      if (k == "--vendor") {
        if (!parse_vendor(v, o.vendor)) {
          std::cerr << "unknown vendor: " << v << "\n";
          return false;
        }
      } else if (k == "--channel") o.channel = v;
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
      else if (k == "--param-id") o.param_id = static_cast<uint16_t>(std::stoul(v, nullptr, 0));
      else if (k == "--param-type") o.param_type = v;
      else if (k == "--param-value") o.param_value = v;
      else if (k == "--param-timeout-ms") o.param_timeout_ms = std::stoi(v);
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

static bool print_state(MotorHandle* motor, const std::string& prefix) {
  MotorState st{};
  if (!ok(motor_handle_get_state(motor, &st), "get_state")) return false;
  if (st.has_value) {
    std::cout << prefix << " pos=" << st.pos << " vel=" << st.vel << " torq=" << st.torq
              << " status=" << static_cast<int>(st.status_code) << " arb=0x" << std::hex
              << st.arbitration_id << std::dec << "\n";
  } else {
    std::cout << prefix << " no feedback yet\n";
  }
  return true;
}

static bool read_robstride_param(MotorHandle* motor, const Options& o) {
  if (o.param_type == "i8") {
    int8_t value = 0;
    if (!ok(motor_handle_robstride_get_param_i8(motor, o.param_id, static_cast<uint32_t>(o.param_timeout_ms), &value),
            "robstride_get_param_i8"))
      return false;
    std::cout << "param 0x" << std::hex << o.param_id << std::dec << " (" << o.param_type
              << ") = " << static_cast<int>(value) << "\n";
    return true;
  }
  if (o.param_type == "u8") {
    uint8_t value = 0;
    if (!ok(motor_handle_robstride_get_param_u8(motor, o.param_id, static_cast<uint32_t>(o.param_timeout_ms), &value),
            "robstride_get_param_u8"))
      return false;
    std::cout << "param 0x" << std::hex << o.param_id << std::dec << " (" << o.param_type
              << ") = " << static_cast<unsigned>(value) << "\n";
    return true;
  }
  if (o.param_type == "u16") {
    uint16_t value = 0;
    if (!ok(motor_handle_robstride_get_param_u16(motor, o.param_id, static_cast<uint32_t>(o.param_timeout_ms), &value),
            "robstride_get_param_u16"))
      return false;
    std::cout << "param 0x" << std::hex << o.param_id << std::dec << " (" << o.param_type
              << ") = " << value << "\n";
    return true;
  }
  if (o.param_type == "u32") {
    uint32_t value = 0;
    if (!ok(motor_handle_robstride_get_param_u32(motor, o.param_id, static_cast<uint32_t>(o.param_timeout_ms), &value),
            "robstride_get_param_u32"))
      return false;
    std::cout << "param 0x" << std::hex << o.param_id << std::dec << " (" << o.param_type
              << ") = " << value << "\n";
    return true;
  }

  float value = 0.0f;
  if (!ok(motor_handle_robstride_get_param_f32(motor, o.param_id, static_cast<uint32_t>(o.param_timeout_ms), &value),
          "robstride_get_param_f32"))
    return false;
  std::cout << "param 0x" << std::hex << o.param_id << std::dec << " (" << o.param_type
            << ") = " << value << "\n";
  return true;
}

static bool write_robstride_param(MotorHandle* motor, const Options& o) {
  try {
    if (o.param_type == "i8") {
      return ok(motor_handle_robstride_write_param_i8(motor, o.param_id,
                                                      static_cast<int8_t>(std::stoi(o.param_value, nullptr, 0))),
                "robstride_write_param_i8");
    }
    if (o.param_type == "u8") {
      return ok(motor_handle_robstride_write_param_u8(motor, o.param_id,
                                                      static_cast<uint8_t>(std::stoul(o.param_value, nullptr, 0))),
                "robstride_write_param_u8");
    }
    if (o.param_type == "u16") {
      return ok(motor_handle_robstride_write_param_u16(motor, o.param_id,
                                                       static_cast<uint16_t>(std::stoul(o.param_value, nullptr, 0))),
                "robstride_write_param_u16");
    }
    if (o.param_type == "u32") {
      return ok(motor_handle_robstride_write_param_u32(motor, o.param_id,
                                                       static_cast<uint32_t>(std::stoul(o.param_value, nullptr, 0))),
                "robstride_write_param_u32");
    }
    return ok(motor_handle_robstride_write_param_f32(motor, o.param_id, std::stof(o.param_value)),
              "robstride_write_param_f32");
  } catch (...) {
    std::cerr << "invalid param value: " << o.param_value << "\n";
    return false;
  }
}

int main(int argc, char** argv) {
  Options o;
  if (!parse_args(argc, argv, o)) {
    if (argc <= 1 || std::strcmp(argv[1], "--help") != 0) print_help();
    return argc > 1 && std::strcmp(argv[1], "--help") == 0 ? 0 : 2;
  }

  if (o.vendor == Vendor::Robstride) {
    if (o.model == "4340") o.model = "rs-00";
    if (o.feedback_id == 0x11) o.feedback_id = 0xFF;
  }

  std::cout << "vendor=" << vendor_name(o.vendor) << " channel=" << o.channel << " model=" << o.model
            << " motor_id=0x" << std::hex << o.motor_id << " feedback_id=0x" << o.feedback_id << std::dec
            << " mode=" << mode_name(o.mode) << "\n";

  MotorController* controller = motor_controller_new_socketcan(o.channel.c_str());
  if (!controller) {
    std::cerr << "create controller failed: " << motor_last_error_message() << "\n";
    return 1;
  }

  MotorHandle* motor = o.vendor == Vendor::Damiao
                           ? motor_controller_add_damiao_motor(controller, o.motor_id, o.feedback_id, o.model.c_str())
                           : motor_controller_add_robstride_motor(controller, o.motor_id, o.feedback_id, o.model.c_str());
  if (!motor) {
    std::cerr << "add motor failed: " << motor_last_error_message() << "\n";
    motor_controller_free(controller);
    return 1;
  }

  if (o.vendor == Vendor::Damiao &&
      (o.mode == Mode::Ping || o.mode == Mode::ReadParam || o.mode == Mode::WriteParam)) {
    std::cerr << "Damiao demo does not support robstride-only modes\n";
    goto out;
  }
  if (o.vendor == Vendor::Robstride && (o.mode == Mode::PosVel || o.mode == Mode::ForcePos)) {
    std::cerr << "RobStride demo supports ping/enable/disable/mit/vel/read-param/write-param\n";
    goto out;
  }

  if (o.mode == Mode::Ping) {
    uint8_t device_id = 0;
    uint8_t responder_id = 0;
    if (!ok(motor_handle_robstride_ping(motor, &device_id, &responder_id), "robstride_ping")) goto out;
    std::cout << "ping ok device_id=" << static_cast<unsigned>(device_id)
              << " responder_id=" << static_cast<unsigned>(responder_id) << "\n";
    (void)print_state(motor, "[state]");
    goto out;
  }

  if (o.mode == Mode::ReadParam) {
    if (!read_robstride_param(motor, o)) goto out;
    (void)print_state(motor, "[state]");
    goto out;
  }

  if (o.mode == Mode::WriteParam) {
    if (!write_robstride_param(motor, o)) goto out;
    if (!read_robstride_param(motor, o)) goto out;
    (void)print_state(motor, "[state]");
    goto out;
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
      if (!print_state(motor, "#" + std::to_string(i))) goto out;
    }

    if (o.dt_ms > 0) std::this_thread::sleep_for(std::chrono::milliseconds(o.dt_ms));
  }

out:
  motor_controller_shutdown(controller);
  motor_handle_free(motor);
  motor_controller_free(controller);
  return 0;
}
