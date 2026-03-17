#include <chrono>
#include <cstdint>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <string>
#include <thread>

#include "motorbridge/motorbridge.hpp"

static uint16_t parse_u16(const std::string& s) {
  return static_cast<uint16_t>(std::stoul(s, nullptr, 0));
}

static std::string hex3(uint16_t v) {
  std::ostringstream oss;
  oss << "0x" << std::hex << std::uppercase << std::setw(3) << std::setfill('0') << v;
  return oss.str();
}

int main(int argc, char** argv) {
  std::string channel = "can0";
  std::string model = "4310";
  uint16_t start_id = 0x001;
  uint16_t end_id = 0x7FF;
  uint16_t feedback_base = 0x10;
  uint32_t timeout_ms = 80;
  uint32_t sleep_ms = 2;
  int verbose = 0;

  for (int i = 1; i < argc; ++i) {
    std::string k = argv[i];
    if (k == "--channel" && i + 1 < argc) channel = argv[++i];
    else if (k == "--model" && i + 1 < argc) model = argv[++i];
    else if (k == "--start-id" && i + 1 < argc) start_id = parse_u16(argv[++i]);
    else if (k == "--end-id" && i + 1 < argc) end_id = parse_u16(argv[++i]);
    else if (k == "--feedback-base" && i + 1 < argc) feedback_base = parse_u16(argv[++i]);
    else if (k == "--timeout-ms" && i + 1 < argc) timeout_ms = static_cast<uint32_t>(std::stoul(argv[++i]));
    else if (k == "--sleep-ms" && i + 1 < argc) sleep_ms = static_cast<uint32_t>(std::stoul(argv[++i]));
    else if (k == "--verbose" && i + 1 < argc) verbose = std::stoi(argv[++i]);
    else if (k == "--help") {
      std::cout
          << "Usage: ./scan_ids_demo [--channel can0] [--model 4310] [--start-id 0x001] [--end-id 0x7FF]\n"
          << "                      [--feedback-base 0x10] [--timeout-ms 80] [--sleep-ms 2] [--verbose 0|1]\n";
      return 0;
    }
  }

  if (start_id < 0x001 || end_id > 0x7FF || start_id > end_id) {
    std::cerr << "invalid range: use 0x001..0x7FF and start <= end\n";
    return 2;
  }

  try {
    motorbridge::Controller ctrl(channel);
    int hits = 0;

    std::cout << "scan start: channel=" << channel << " model=" << model << " range=" << hex3(start_id)
              << ".." << hex3(end_id) << " feedback-base=" << hex3(feedback_base)
              << " timeout-ms=" << timeout_ms << "\n";

    for (uint16_t mid = start_id; mid <= end_id; ++mid) {
      uint16_t fid = static_cast<uint16_t>(feedback_base + (mid & 0x0F));
      bool ok = false;

      try {
        auto m = ctrl.add_damiao_motor(mid, fid, model);
        try {
          (void)m.get_register_u32(motorbridge::RID_CTRL_MODE, timeout_ms);
          ok = true;
        } catch (const std::exception& e) {
          if (verbose) {
            std::cerr << "[probe-reg-miss] motor-id=" << hex3(mid) << " feedback-id=" << hex3(fid)
                      << " err=" << e.what() << "\n";
          }
        }
        if (!ok) {
          try {
            m.enable();
            m.request_feedback();
            for (int t = 0; t < 3; ++t) {
              ctrl.poll_feedback_once();
              if (m.get_state().has_value()) {
                ok = true;
                break;
              }
              std::this_thread::sleep_for(std::chrono::milliseconds(5));
            }
          } catch (const std::exception& e) {
            if (verbose) {
              std::cerr << "[probe-enable-miss] motor-id=" << hex3(mid) << " feedback-id=" << hex3(fid)
                        << " err=" << e.what() << "\n";
            }
          }
        }
      } catch (const std::exception& e) {
        if (verbose) {
          std::cerr << "[add-motor-miss] motor-id=" << hex3(mid) << " feedback-id=" << hex3(fid)
                    << " err=" << e.what() << "\n";
        }
      }

      if (ok) {
        std::cout << "[hit] motor-id=" << hex3(mid) << " feedback-id=" << hex3(fid) << "\n";
        std::cout.flush();
        ++hits;
      }

      if (sleep_ms > 0) std::this_thread::sleep_for(std::chrono::milliseconds(sleep_ms));
      if (mid == 0x7FF) break;
    }

    std::cout << "scan done, hits=" << hits << "\n";
    return 0;
  } catch (const std::exception& e) {
    std::cerr << "fatal: " << e.what() << "\n";
    return 1;
  }
}

