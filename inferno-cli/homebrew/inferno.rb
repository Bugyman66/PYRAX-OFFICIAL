# Homebrew formula for Inferno CLI
# To install: brew tap pyrax-chain/tap && brew install inferno

class Inferno < Formula
  desc "Terminal-based node management for PYRAX blockchain"
  homepage "https://github.com/PYRAX-Chain/PYRAX-OFFICIAL"
  version "0.2.47"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases/download/cli-v#{version}/inferno-cli-#{version}-darwin-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    end
    on_intel do
      url "https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases/download/cli-v#{version}/inferno-cli-#{version}-darwin-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases/download/cli-v#{version}/inferno-cli-#{version}-linux-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases/download/cli-v#{version}/inferno-cli-#{version}-linux-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  def install
    bin.install "inferno"
    
    # Generate and install shell completions
    generate_completions_from_executable(bin/"inferno", "completions")
  end

  def post_install
    # Create config directory
    (var/"lib/inferno").mkpath
  end

  def caveats
    <<~EOS
      To get started with Inferno CLI:
        
        1. Initialize your first node:
           inferno init
        
        2. Start your node:
           inferno start
        
        3. View logs:
           inferno logs --follow
        
        4. Open dashboard:
           inferno dashboard --open
      
      For more information, visit:
        https://github.com/PYRAX-Chain/PYRAX-OFFICIAL
    EOS
  end

  test do
    assert_match "Inferno CLI", shell_output("#{bin}/inferno --version")
  end
end
