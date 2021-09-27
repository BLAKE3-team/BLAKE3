{ pkgs, system }:
let
  headerFile = ./blake3.h;
  inherit (pkgs) stdenv;
  src = ./.;
  buildCLib = pkgs.lib.makeOverridable
    # Wrap object files in an archive for static linking
    ({ static ? false
     , libExtension ? if static then "a" else "so"
     , libName ? "libblake3.${libExtension}"
     , cc ? stdenv.cc
     , ccOptions ? [ ]
     , debug ? false
     }:
      let
        lib = pkgs.lib;
        spaceJoin = lib.concatStringsSep " ";
        name = libName;
        nativeImplementations =
          if system == "x86_64-linux"
          then [ "blake3_sse2_x86-64_unix.S" "blake3_sse41_x86-64_unix.S" "blake3_avx2_x86-64_unix.S" "blake3_avx512_x86-64_unix.S" ]
          else [ ];
        sourceFiles = [ "blake3.c" "blake3_dispatch.c" "blake3_portable.c" ] ++ nativeImplementations;
        warningOptions = [ "-Wall" "-Werror" "-Wextra" "-pedantic" ];
        commonCCOptions = spaceJoin (warningOptions
          ++ [ "-O3" (if debug then "-ggdb" else "") ] ++ ccOptions);
        buildSteps =
          if static then
            [
              "${cc}/bin/cc ${commonCCOptions} -c ${spaceJoin sourceFiles}"
              "ar rcs ${libName} *.o"

            ] else
            [
              "${cc}/bin/cc ${commonCCOptions} -shared -fPIC -o ${libName} ${spaceJoin sourceFiles}"
            ];
        drvArgs = {
          inherit name src;
          buildInputs = with pkgs; [ cc clib ];
          buildPhase = pkgs.lib.concatStringsSep "\n" buildSteps;
          installPhase = ''
            mkdir -p $out
            cp ${libName} $out
          '';
        } // (if debug then {
          NIX_DEBUG = 1;
        } else { });
      in
      pkgs.stdenv.mkDerivation drvArgs);
  # Add additional properties
  cLib = (args:
    let
      self = buildCLib args;
    in
    self // {
      debug = self.override { debug = true; };
    });
  staticLib = cLib {
    static = true;
  };
  dynamicLib = cLib {
    static = false;
  };
in
staticLib // {
  inherit cLib dynamicLib staticLib headerFile;
}
