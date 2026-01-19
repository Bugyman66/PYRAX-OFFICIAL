class Inferno < Formula
  desc "Terminal-based node management for PYRAX blockchain"
  homepage "https://github.com/PYRAX-Chain/PYRAX-OFFICIAL"
  version "0.2.48"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases/download/cli-v#{version}/inferno-cli-#{version}-darwin-aarch64.tar.gz"
      sha256 "0fea0b0e9802b1ffc350c068f9c3c18c8c1a543eb9fe3ce3871c74e1331f2a16"
    end
    on_intel do
      url "https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases/download/cli-v#{version}/inferno-cli-#{version}-darwin-x86_64.tar.gz"
      sha256 "d5e4d91e596e227acf349264b2c915496dcea9a7235598ef987a1e93f88a7ecb"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases/download/cli-v#{version}/inferno-cli-#{version}-linux-x86_64.tar.gz"
      sha256 "690f09ad3fecb825690aaa663bf5b2f2feeded4e5b62cf02cfb8addb684a607b"
    end
  end

  def install
    bin.install "inferno"
    generate_completions_from_executable(bin/"inferno", "completions")
  end

  test do
    assert_match "Inferno CLI", shell_output("#{bin}/inferno --version")
  end
end
