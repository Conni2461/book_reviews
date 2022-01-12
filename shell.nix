let pkgs = import <nixpkgs> { };
in
let
  packageOverrides = pkgs.callPackage ./python-packages.nix { };
  python = pkgs.python39.override {
    packageOverrides = self: super: (
      (packageOverrides self super) // {
        scipy = super.scipy;
        scikit-learn = super.scikit-learn;
        numpy = super.numpy;
        six = super.six;
        lxml = super.lxml;
      }
    );
  };
  pythonWithPackages = python.withPackages (ps: [
    ps.SQLAlchemy
    ps.requests
    ps.selectorlib
    ps.numpy
    ps.scikit-learn
  ]);
in
pkgs.mkShell {
  nativeBuildInputs = [
    pythonWithPackages
  ];
  shellHook = ''
    exec zsh
  '';
}
