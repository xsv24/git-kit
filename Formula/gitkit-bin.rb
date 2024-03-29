class GitKitBin < Formula
    version '0.0.22-pre1'
    desc "git cli containing templates & utilities."
    homepage "https://github.com/xsv24/git-kit"
  
    if OS.mac?
        # "https://github.com/xsv24/git-kit/releases/download/0.0.22-pre1/git-kit--aarch64-apple-darwin.tar.gz"
        url "https://github.com/xsv24/git-kit/releases/download/#{version}/git-kit-aarch64-apple-darwin.tar.gz"
        sha256 "6c69c2d7e34e0fbbe08461fa1ec6bf4c20e2a2cca9c6aca70281e4b580b8827d"
    elsif OS.linux?
        url "https://github.com/xsv24/git-kit/releases/download/#{version}/git-kit-#{version}-x86_64-unknown-linux-musl.tar.gz"
        sha256 "6c69c2d7e34e0fbbe08461fa1ec6bf4c20e2a2cca9c6aca70281e4b580b8827d"
    end
  
    # inkan
    conflicts_with "git-kit"
  
    def install
      ohai "Installing MyFormula"
      bin.install "git-kit"
      # Install the configuration file to $HOME/.config/myapp/config.conf
      (etc/"git-kit").install "../templates/conventional.yml"
      (etc/"git-kit").install "../templates/default.yml"
      # man1.install "doc/rg.1"
  
      # bash_completion.install "complete/rg.bash"
      # zsh_completion.install "complete/_rg"
    end

    test do
      # `test do` will create, run in and delete a temporary directory.
      #
      # This test will fail and we won't accept that! For Homebrew/homebrew-core
      # this will need to be a test that verifies the functionality of the
      # software. Run the test with `brew test git-kit`. Options passed
      # to `brew install` such as `--HEAD` also need to be provided to `brew test`.
      #
      # The installed folder is not in the path, so use the entire path to any
      # executables being tested: `system "#{bin}/program", "do", "something"`.
      system "#{bin}/git-kit"
    end
  end