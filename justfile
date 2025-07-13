install:
  cargo build --release
  sudo install -Dm755 target/release/aletheia /usr/bin
  sudo install -Dm644 resources/linux/completions/aletheia.bash /usr/share/bash-completion/completions/aletheia
  sudo install -Dm644 resources/linux/completions/aletheia.fish /usr/share/fish/vendor_completions.d
  sudo install -Dm644 resources/linux/moe.spencer.Aletheia.desktop /usr/share/applications
  sudo install -Dm644 resources/logo/moe.spencer.Aletheia.png /usr/share/icons/hicolor/512x512/apps
  sudo install -Dm644 resources/linux/moe.spencer.Aletheia.metainfo.xml /usr/share/metainfo

install_flatpak:
  flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
  flatpak-builder --force-clean --user --install-deps-from=flathub --repo=repo --install builddir resources/flatpak/moe.spencer.Aletheia.yaml

uninstall:
  sudo rm /usr/bin/aletheia
  sudo rm /usr/share/bash-completion/completions/aletheia
  sudo rm /usr/share/fish/vendor_completions.d/aletheia.fish
  sudo rm /usr/share/applications/moe.spencer.Aletheia.desktop
  sudo rm /usr/share/icons/hicolor/512x512/apps/moe.spencer.Aletheia.png
  sudo rm /usr/share/metainfo/moe.spencer.Aletheia.metainfo.xml

uninstall_flatpak:
  flatpak uninstall moe.spencer.Aletheia

generate_translations:
  find -name \*.slint | xargs slint-tr-extractor -o aletheia.pot

update_translations:
  find -name \*.slint | xargs slint-tr-extractor -o aletheia.pot
  find ui/locale/*/LC_MESSAGES -name "aletheia.po" | xargs -I {} msgmerge -U {} aletheia.pot
