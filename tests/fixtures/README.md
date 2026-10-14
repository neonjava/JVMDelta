# GC log fixtures

- `jdk17-g1-synthetic.log`: synthetic JDK 17-style G1 examples: young, full, remark, cleanup, and a `gc,start` line. Created for parser tests; not captured from JDK 17.
- `jdk21-g1-synthetic.log`: synthetic JDK 21-style G1 examples with wall-clock and uptime decorations, K/G units, and a detail record. Created for parser tests; not captured from JDK 21.
- `malformed-g1-synthetic.log`: synthetic malformed size-unit case.
- `jdk25-g1-real.log`: selected unchanged lines from a local OpenJDK 25.0.3 G1 run on Fedora. It contains one young summary, one full summary, a start record, and three heap detail records. Pause timings and heap quantities are specific to that run; do not compare them as performance evidence.

To reproduce a similar JDK 25 log, save this as `/tmp/JvmDeltaGcFixture.java`:

```java
public class JvmDeltaGcFixture {
    public static void main(String[] args) {
        for (int i = 0; i < 100; i++) {
            byte[] bytes = new byte[1024 * 1024];
            bytes[0] = 1;
        }
        System.gc();
    }
}
```

Then run:

```sh
java -Xms32m -Xmx32m -XX:+UseG1GC '-Xlog:gc*:file=/tmp/jvmdelta-gc.log:uptime,level,tags' /tmp/JvmDeltaGcFixture.java
```

The exact event IDs, heap values, and timing vary between runs. The fixture has no application data or personal paths.
