package network.icon.intent;

import java.math.BigInteger;

import network.icon.intent.constants.Constant;
import network.icon.intent.structs.PermitTransferFrom;
import network.icon.intent.utils.PermitTransferFromData;
import network.icon.intent.utils.SignatureTransferDetailsData;
import score.Address;
import score.BranchDB;
import score.Context;
import score.DictDB;
import score.VarDB;
import score.annotation.External;

public class Permit2 {
        private final VarDB<String> _token_permission_type = Context.newVarDB(Constant._TOKEN_PERMISSION_TYPE,
                        String.class);
        private final BranchDB<Address, DictDB<BigInteger, BigInteger>> nonceBitmap = Context.newBranchDB(
                        Constant.NONCE_BITMAP,
                        BigInteger.class);
        private final VarDB<byte[]> _hashed_name = Context.newVarDB(Constant._HASHED_NAME, byte[].class);
        private final VarDB<byte[]> _type_hash = Context.newVarDB(Constant._TYPE_HASH, byte[].class);
        private final VarDB<byte[]> _token_permission_typehash = Context.newVarDB(Constant._TOKEN_PERMISSION_TYPEHASH,
                        byte[].class);
        private final VarDB<String> _pERMIT_tRANSFER_fROM_wITNESS_tYPEHASH_sTUB = Context.newVarDB(
                        Constant._pERMIT_tRANSFER_fROM_wITNESS_tYPEHASH_sTUB,
                        String.class);
        private final VarDB<BigInteger> _chain_id = Context.newVarDB(Constant._CHAIN_ID, BigInteger.class);
        private final VarDB<byte[]> _CACHED_DOMAIN_SEPARATOR = Context.newVarDB(Constant._CACHED_DOMAIN_SEPARATOR,
                        byte[].class);
        private final VarDB<BigInteger> _CACHED_CHAIN_ID = Context.newVarDB(Constant._CACHED_CHAIN_ID,
                        BigInteger.class);
        public static final VarDB<Address> owner = Context.newVarDB(Constant.OWNER, Address.class);

        final byte[] _HASHED_NAME = Context.hash("keccak-256", "Permit2".getBytes());
        final byte[] _TYPE_HASH = Context.hash("keccak-256",
                        ("EIP712Domain(string name,uint256 chainId,address verifyingContract)").getBytes());

        byte[] _TOKEN_PERMISSIONS_TYPEHASH = Context.hash("keccak-256",
                        ("TokenPermissions(address token,uint256 amount)".getBytes()));

        public Permit2() {
                _chain_id.set(BigInteger.valueOf(1));
                _hashed_name.set(_HASHED_NAME);
                _token_permission_type.set(Constant.TOKEN_PERMISSIONS_TYPE);
                _type_hash.set(_TYPE_HASH);
                _token_permission_typehash.set(_TOKEN_PERMISSIONS_TYPEHASH);
                _CACHED_CHAIN_ID.set(_chain_id.get());
                _pERMIT_tRANSFER_fROM_wITNESS_tYPEHASH_sTUB.set(Constant._PERMIT_TRANSFER_FROM_WITNESS_TYPEHASH_STUB);
                _CACHED_DOMAIN_SEPARATOR.set(_buildDomainSeparator(_type_hash.get(), _hashed_name.get()));
                owner.set(Context.getCaller());
        }

        @External
        public void permitWitnessTransferFrom(
                        PermitTransferFromData permit,
                        SignatureTransferDetailsData transferDetails,
                        Address owner,
                        byte[] witness,
                        String witnessTypeString,
                        byte[] signature) {
                _permitTransferFrom(permit, transferDetails, owner, hashWithWitness(permit, witness, witnessTypeString),
                                signature);
        }

        void _permitTransferFrom(
                        PermitTransferFromData permit,
                        SignatureTransferDetailsData transferDetails,
                        Address owner,
                        byte[] dataHash,
                        byte[] signature) {
                BigInteger requestedAmount = transferDetails.requestedAmount;
                Context.require(BigInteger.valueOf(Context.getBlockTimestamp()).compareTo(permit.deadline) < 0,
                                "Signature Expired");
                Context.require(requestedAmount.compareTo(permit.permitted.amount) < 0, "InvalidAmount");

                _useUnorderedNonce(owner, permit.nonce);
                Context.verifySignature("ed25519", dataHash, signature, owner.toByteArray());
                Context.call(permit.permitted.token, "transfer", transferDetails.to, requestedAmount);
        }

        BigInteger[] bitmapPositions(BigInteger nonce) {
                BigInteger wordPos = nonce.shiftRight(8);
                BigInteger bitPos = nonce.and(BigInteger.valueOf(255));

                return new BigInteger[] { wordPos, bitPos };
        }

        void _useUnorderedNonce(Address from, BigInteger nonce) {
                BigInteger[] positions = bitmapPositions(nonce);
                BigInteger wordPos = positions[0];
                BigInteger bitPos = positions[1];

                BigInteger bit = BigInteger.ONE.shiftLeft(bitPos.intValue());

                DictDB<BigInteger, BigInteger> innerDict = nonceBitmap.at(from);

                BigInteger currentValue = innerDict.get(wordPos);
                if (currentValue == null) {
                        currentValue = BigInteger.ZERO;
                }

                BigInteger flipped = currentValue.xor(bit);
                innerDict.set(wordPos, flipped);
                Context.require(!(flipped.and(bit).equals(BigInteger.ZERO)), "Invalid Nonce");
        }

        byte[] hashWithWitness(
                        PermitTransferFromData permit,
                        byte[] witness,
                        String witnessTypeString) {

                byte[] concatenatedData1 = new byte[_pERMIT_tRANSFER_fROM_wITNESS_tYPEHASH_sTUB.get().getBytes().length
                                + witnessTypeString.getBytes().length];
                System.arraycopy(_pERMIT_tRANSFER_fROM_wITNESS_tYPEHASH_sTUB.get(), 0, concatenatedData1, 0,
                                _pERMIT_tRANSFER_fROM_wITNESS_tYPEHASH_sTUB.get().getBytes().length);
                System.arraycopy(witnessTypeString.getBytes(), 0, concatenatedData1,
                                _pERMIT_tRANSFER_fROM_wITNESS_tYPEHASH_sTUB.get().getBytes().length,
                                witnessTypeString.getBytes().length);

                byte[] typeHashBytes = Context.hash("keccak-256", concatenatedData1);

                byte[] tokenPermissionsHash = _hashTokenPermissions(permit);

                byte[] concatenatedData = new byte[typeHashBytes.length + tokenPermissionsHash.length
                                + Context.getCaller().toString().getBytes().length
                                + permit.nonce.toString().getBytes().length
                                + permit.deadline.toString().getBytes().length
                                + witness.length];
                System.arraycopy(typeHashBytes, 0, concatenatedData, 0, typeHashBytes.length);
                System.arraycopy(tokenPermissionsHash, 0, concatenatedData, typeHashBytes.length,
                                tokenPermissionsHash.length);
                System.arraycopy(Context.getCaller().toString().getBytes(), 0, concatenatedData,
                                typeHashBytes.length + tokenPermissionsHash.length,
                                Context.getCaller().toString().getBytes().length);
                System.arraycopy(permit.nonce.toString().getBytes(), 0, concatenatedData,
                                typeHashBytes.length + tokenPermissionsHash.length
                                                + Context.getCaller().toString().getBytes().length,
                                permit.nonce.toString().getBytes().length);
                System.arraycopy(permit.deadline.toString().getBytes(), 0, concatenatedData,
                                typeHashBytes.length + tokenPermissionsHash.length
                                                + Context.getCaller().toString().getBytes().length
                                                + permit.nonce.toString().getBytes().length,
                                permit.deadline.toString().getBytes().length);
                System.arraycopy(witness, 0, concatenatedData, typeHashBytes.length + tokenPermissionsHash.length
                                + Context.getCaller().toString().getBytes().length
                                + permit.nonce.toString().getBytes().length
                                + permit.deadline.toString().getBytes().length,
                                witness.length);

                return Context.hash("keccak-256", concatenatedData);
        }

        public byte[] _hashTokenPermissions(PermitTransferFromData permitted) {
                PermitTransferFrom permit = new PermitTransferFrom(permitted.permitted, permitted.nonce,
                                permitted.deadline);
                byte[] permittedBytes = permit.toBytes();

                byte[] concatenatedData = new byte[_token_permission_typehash.get().length + permittedBytes.length];
                System.arraycopy(_token_permission_typehash.get(), 0, concatenatedData, 0,
                                _token_permission_typehash.get().length);
                System.arraycopy(permittedBytes, 0, concatenatedData, _token_permission_typehash.get().length,
                                permittedBytes.length);

                return Context.hash("keccak-256", concatenatedData);
        }

        public byte[] DOMAIN_SEPARATOR() {
                return _chain_id.get() == _CACHED_CHAIN_ID
                                ? _CACHED_DOMAIN_SEPARATOR.get()
                                : _buildDomainSeparator(_type_hash.get(), _hashed_name.get());
        }

        public byte[] _buildDomainSeparator(byte[] typeHash, byte[] nameHash) {
                byte[] chainId = _chain_id.get().toByteArray();
                byte[] contractAddress = Context.getAddress().toByteArray();

                byte[] concatenatedData = new byte[typeHash.length + nameHash.length + chainId.length
                                + contractAddress.length];
                System.arraycopy(typeHash, 0, concatenatedData, 0, typeHash.length);
                System.arraycopy(nameHash, 0, concatenatedData, typeHash.length, nameHash.length);
                System.arraycopy(chainId, 0, concatenatedData, typeHash.length + nameHash.length, chainId.length);
                System.arraycopy(contractAddress, 0, concatenatedData,
                                typeHash.length + nameHash.length + chainId.length,
                                contractAddress.length);

                return Context.hash("keccak-256", concatenatedData);
        }

        byte[] _hashTypedData(byte[] dataHash) {
                byte[] domainSeparator = DOMAIN_SEPARATOR();
                byte[] concatenatedData = new byte[2 + domainSeparator.length + dataHash.length];
                concatenatedData[0] = 0x19;
                concatenatedData[1] = 0x01;
                System.arraycopy(domainSeparator, 0, concatenatedData, 2, domainSeparator.length);
                System.arraycopy(dataHash, 0, concatenatedData, 2 + domainSeparator.length, dataHash.length);

                return Context.hash("keccak-256", concatenatedData);
        }

        @External(readonly = true)
        public BigInteger getChainId() {
                return _chain_id.get();
        }

        @External
        public void setChainId(BigInteger chainId) {
                OnlyOwner();
                _chain_id.set(chainId);
        }

        static void OnlyOwner() {
                Context.require(owner.get().equals(Context.getCaller()), "Not Owner");
        }
}
