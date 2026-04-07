class AmandaWatch < Formula
  desc "Process and resource monitoring for Amanda OS"
  homepage "https://github.com/corinoah1013/amanda-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/corinoah1013/amanda-cli/releases/download/v0.1.0/amanda-watch-macos-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X64"
    end
    on_arm do
      url "https://github.com/corinoah1013/amanda-cli/releases/download/v0.1.0/amanda-watch-macos-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/corinoah1013/amanda-cli/releases/download/v0.1.0/amanda-watch-linux-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
    on_arm do
      url "https://github.com/corinoah1013/amanda-cli/releases/download/v0.1.0/amanda-watch-linux-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
  end

  def install
    bin.install "amanda-watch"
  end

  test do
    system "#{bin}/amanda-watch", "--version"
  end
end
