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

import java.math.BigInteger;

import score.ByteArrayObjectWriter;
import score.Context;
import score.ObjectReader;
import score.ObjectWriter;

public class CSMessageResult {
    public static final int SUCCESS = 1;
    public static final int FAILURE = 0;

    private final BigInteger sn;
    private final int code;
    private final byte[] msg;

    public CSMessageResult(BigInteger sn, int code, byte[] msg) {
        this.sn = sn;
        this.code = code;
        this.msg = msg;
    }

    public BigInteger getSn() {
        return sn;
    }

    public int getCode() {
        return code;
    }

    public byte[] getMsg() {
        return msg;
    }

    public static void writeObject(ObjectWriter w, CSMessageResult m) {
        w.beginList(3);
        w.write(m.sn);
        w.write(m.code);
        w.writeNullable(m.msg);
        w.end();
    }

    public static CSMessageResult readObject(ObjectReader r) {
        r.beginList();
        BigInteger sn = r.readBigInteger();
        int code = r.readInt();
        byte[] msg = null;
        if (r.hasNext()) {
            msg = r.readNullable(byte[].class);
        }

        r.end();
        return new CSMessageResult(sn, code, msg);
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
