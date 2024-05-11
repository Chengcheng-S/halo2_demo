Halo2 examples

run:
```shell
cargo run --release
```
generate circuit layout image
```shell
cargo run --release --features dev-graph

run all test && generate circuit layout images
```

```shell
make help

make testgraph
```

circuit layouts:
在halo2生成的电路图中： 红色代表 advice column，蓝色为fix， 白色部分则是instance/public inputs，绿色则是region的调用部分。