## Build and generate example blog

```shell
cargo run # build the site
nix-shell -p tailwindcss_4 --run 'tailwindcss --cwd ../src' > target/example-site/style.css # generate css
python3 -m http.server --directory ./target 4343
```

Point your web browser to [http://localhost:4343/example-site](http://localhost:4343/example-site)
