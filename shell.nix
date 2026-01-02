{ headless ? false }:

(import ./. { inherit headless; }).shell
