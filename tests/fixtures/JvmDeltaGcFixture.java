public class JvmDeltaGcFixture {
    public static void main(String[] args) {
        for (int i = 0; i < 100; i++) {
            byte[] bytes = new byte[1024 * 1024];
            bytes[0] = 1;
        }
        System.gc();
    }
}
