# i3status-nix-update-widget

 An i3status-rust widget to inform you of the age of your nix flake lockfile. Only intended for use with a nix flake, as it is patched at build time with the date of the most recent git commit in that lockfile.

## Installation

put something like
``` nix
programs.i3status-rust.bars.<name>.blocks = [{
    block = "custom";
    command = "${nix-update-widget.packages.${system}.default.override {
        flakelock = ./flake.lock;
    }}/bin/i3status-nix-update-widget";
    interval = 3000;
}];
```
in your home-manager config.

## License
This readme based on [makeareadme](https://www.makeareadme.com/) 
A license can be chosen at [choosealicense](https://choosealicense.com/)
[MIT](https://choosealicense.com/licenses/mit/)

