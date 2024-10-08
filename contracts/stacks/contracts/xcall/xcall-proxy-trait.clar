(use-trait xcall-impl-trait .xcall-impl-trait.xcall-impl-trait)

(define-trait xcall-proxy-trait
  (
    (get-current-implementation () (response principal bool))
    (get-current-proxy () (response (optional principal) uint))
    
    (is-current-implementation (principal) (response bool uint))

    (send-call ((string-ascii 64) (buff 2048) <xcall-impl-trait>) (response uint uint))

    (send-call-message ((string-ascii 64) (buff 2048) (optional (buff 1024)) (optional (list 10 (string-ascii 64))) (optional (list 10 (string-ascii 64))) <xcall-impl-trait>) (response uint uint))

    (execute-call (uint (buff 2048) <xcall-impl-trait>) (response bool uint))

    (execute-rollback (uint <xcall-impl-trait>) (response bool uint))

    (verify-success (uint <xcall-impl-trait>) (response bool uint))

    (handle-message ((string-ascii 64) (buff 2048) <xcall-impl-trait>) (response bool uint))

    (handle-error (uint <xcall-impl-trait>) (response bool uint))

    (set-admin (principal <xcall-impl-trait>) (response bool uint))

    (set-protocol-fee-handler (principal <xcall-impl-trait>) (response bool uint))

    (set-protocol-fee (uint <xcall-impl-trait>) (response bool uint))

    (set-default-connection ((string-ascii 64) (string-ascii 64) <xcall-impl-trait>) (response bool uint))

    (get-network-address (<xcall-impl-trait>) (response (string-ascii 129) uint))

    (get-network-id (<xcall-impl-trait>) (response (string-ascii 64) uint))

    (get-protocol-fee (<xcall-impl-trait>) (response uint uint))

    (get-fee ((string-ascii 64) bool (optional (list 10 (string-ascii 64))) <xcall-impl-trait>) (response uint uint))
  )
)