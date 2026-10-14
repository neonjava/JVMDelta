# GC log fixtures

The Rust test suite reads these text files and does not need Java.

- `jdk17-g1-real.log`: five selected, unchanged lines from Eclipse Temurin 17.0.20.1+1 G1 output: young start, heap detail, young summary, full start, and full summary.
- `jdk21-g1-real.log`: the same categories from Eclipse Temurin 21.0.12.1+1 G1 output.
- `jdk25-g1-real.log`: selected, unchanged lines from a local Fedora OpenJDK 25.0.3 G1 run.
- `jdk17-g1-synthetic.log`: synthetic edge-case examples for young, full, remark, cleanup, and `gc,start`. It is not a recording from JDK 17.
- `jdk21-g1-synthetic.log`: synthetic wall-clock decoration, K/G unit, and GC detail examples. It is not a recording from JDK 21.
- `malformed-g1-synthetic.log`: synthetic invalid size-unit case.

The real 17/21 excerpts were generated on 2026-10-14 using the checked-in [`JvmDeltaGcFixture.java`](JvmDeltaGcFixture.java) allocation workload. The Temurin JRE archives were downloaded to `/tmp` from the [Adoptium binary API](https://adoptium.net/installation/ci-scripts/) and verified against the API's SHA-256 metadata. The archive SHA-256 values were:

- Temurin 17 JRE: `0b2b640e3046b64c8ec504de0ab9d91bb5610182bda21fad454681ce54d45a62`
- Temurin 21 JRE: `2413149700df0f7d440500a84a8f764c535f21e5a5e87d38328b64eec2c5b500`

A local JDK 17 or 21 can reproduce comparable output without changing the system configuration. From the repository root, replace `JAVA17_HOME` and `JAVA21_HOME` with local paths:

```sh
mkdir -p /tmp/jvmdelta-fixture-classes
javac --release 17 -d /tmp/jvmdelta-fixture-classes tests/fixtures/JvmDeltaGcFixture.java
"$JAVA17_HOME/bin/java" -Xms32m -Xmx32m -XX:+UseG1GC '-Xlog:gc*:file=/tmp/jvmdelta-jdk17-gc.log:uptime,level,tags' -cp /tmp/jvmdelta-fixture-classes JvmDeltaGcFixture
"$JAVA21_HOME/bin/java" -Xms32m -Xmx32m -XX:+UseG1GC '-Xlog:gc*:file=/tmp/jvmdelta-jdk21-gc.log:uptime,level,tags' -cp /tmp/jvmdelta-fixture-classes JvmDeltaGcFixture
```

A JDK 17 or newer compiler can compile the workload. The archive used for validation provided JREs; the installed JDK 25 compiler built the source with `--release 17`. Exact event IDs, heap values, and timings vary between runs. The fixtures contain no application data or personal paths and must not be interpreted as performance benchmarks.
