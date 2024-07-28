Build Automation:
Build is automated through GitHub Actions and the script is found in ./github/workflows/rust.yml
The script automatically runs when changes are made to the main branch.

To build manually, you need to ensure rust is installed on your system.
It can be installed via the rustup executable (or via curl) found here: https://www.rust-lang.org/tools/install

Once setup, the build process is as follows:

1. Navigate to the root project directory in a terminal (i.e. C:\Users\User\Desktop\RustWeather)
2. (Optional) Enter the following command to build a development build: cargo build
3. (Optional) Enter the following command to run unit tests: cargo test
4. Enter the following command to create a release builld: cargo build --release

The executable can then be found in RustWeather\target\release\ (or RustWeather\target\debug\ for the dev build)