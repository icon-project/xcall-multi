package network.icon.intent.constants;

import java.math.BigInteger;

public class Constant {
    public static final BigInteger FILL = new BigInteger("1");
    public static final BigInteger CANCEL = new BigInteger("2");

    // Generalized Connection Variables
    public final static String RECEIPTS = "receipts";
    public final static String RELAY_ADDRESS = "relayAddress";
    public final static String CONN_SN = "connSn";

    // Permit Variables
    public final static String TOKEN_PERMISSIONS_TYPE = "TokenPermissions(address token,uint256 amount)";
    public final static String _TOKEN_PERMISSION_TYPE = "_token_permission_type";
    public final static String NONCE_BITMAP = "nonceBitmap";
    public final static String _HASHED_NAME = "_hashed_name";
    public final static String _TYPE_HASH = "_type_hash";
    public final static String _TOKEN_PERMISSION_TYPEHASH = "_token_permission_typehash";
    public final static String _PERMIT_TRANSFER_FROM_WITNESS_TYPEHASH_STUB = "PermitWitnessTransferFrom(TokenPermissions permitted,address spender,uint256 nonce, uint256 deadline,";
    public final static String _pERMIT_tRANSFER_fROM_wITNESS_tYPEHASH_sTUB = "_pERMIT_tRANSFER_fROM_wITNESS_tYPEHASH_sTUB";
    public final static String _CHAIN_ID = "_chain_id";
    public final static String _CACHED_DOMAIN_SEPARATOR = "_CACHED_DOMAIN_SEPARATOR";
    public final static String _CACHED_CHAIN_ID = "_CACHED_CHAIN_ID";

    // Intent Variables
    public final static String DEPOSIT_ID = "depositId";
    public final static String NETWORK_ID = "networkId";
    public final static String PROTOCOL_FEE = "protocolFee";
    public final static String FEE_HANDLER = "feeHandler";
    public final static String OWNER = "owner";
    public final static String NATIVE_ADDRESS = "nativeAddress";
    public final static String DEPOSIT = "deposit";
    public final static String ORDERS = "orders";
    public final static String FINISHED_ORDERS = "finishedOrders";
    public final static String PERMIT = "permit";
}
