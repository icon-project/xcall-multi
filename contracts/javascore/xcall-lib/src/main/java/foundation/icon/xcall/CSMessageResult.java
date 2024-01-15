/*
 * Copyright 2022 ICON Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package foundation.icon.xcall;

import score.ByteArrayObjectWriter;
import score.Context;
import score.ObjectReader;
import score.ObjectWriter;

import java.math.BigInteger;

public class CSMessageResult {
    public static final int SUCCESS = 1;
    public static final int FAILURE = 0;

    private final BigInteger sn;
    private final int code;

    public CSMessageResult(BigInteger sn, int code) {
        this.sn = sn;
        this.code = code;
    }

    public BigInteger getSn() {
        return sn;
    }

    public int getCode() {
        return code;
    }

    public static void writeObject(ObjectWriter w, CSMessageResult m) {
        w.beginList(2);
        w.write(m.sn);
        w.write(m.code);
        w.end();
    }

    public static CSMessageResult readObject(ObjectReader r) {
        r.beginList();
        CSMessageResult m = new CSMessageResult(
                r.readBigInteger(),
                r.readInt()
        );
        r.end();
        return m;
    }

    public byte[] toBytes() {
        ByteArrayObjectWriter writer = Context.newByteArrayObjectWriter("RLPn");
        CSMessageResult.writeObject(writer, this);
        return writer.toByteArray();
    }

    public static CSMessageResult fromBytes(byte[] bytes) {
        ObjectReader reader = Context.newByteArrayObjectReader("RLPn", bytes);
        return readObject(reader);
    }
}
