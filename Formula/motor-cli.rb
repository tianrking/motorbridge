class MotorCli < Formula
  desc "Unified multi-vendor CAN motor command line tool"
  homepage "https://github.com/tianrking/motorbridge"
  version "0.0.0"
  url "https://github.com/tianrking/motorbridge/releases/download/v0.0.0/motor-cli-v0.0.0-macos-arm64.tar.gz"
  sha256 "REPLACE_WITH_REAL_SHA256"
  license "MIT"

  depends_on :macos

  def install
    bin.install "bin/motor_cli" => "motor_cli"
    prefix.install "BUILD_INFO.txt" if File.exist?("BUILD_INFO.txt")
  end

  test do
    assert_match "motor_cli", shell_output("#{bin}/motor_cli --help")
  end
end
