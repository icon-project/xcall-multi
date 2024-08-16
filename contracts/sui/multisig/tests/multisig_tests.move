
#[test_only]
module multisig::multisig_tests {
    // uncomment this line to import the module
    use multisig::multisig;

    const ENotImplemented: u64 = 0;

    #[test]
    fun test_create_multisig() {
       // let pub_keys=[];
    }

    #[test, expected_failure(abort_code = ::multisig::multisig_tests::ENotImplemented)]
    fun test_multisig_fail() {
        abort ENotImplemented
    }
}

