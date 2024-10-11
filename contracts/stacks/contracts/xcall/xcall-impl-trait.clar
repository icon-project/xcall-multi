(define-trait xcall-impl-trait
  (
    (send-call ((string-ascii 128) (buff 2048)) (response uint uint))
    (send-call-message ((string-ascii 128) (buff 2048) (optional (buff 1024)) (optional (list 10 (string-ascii 128))) (optional (list 10 (string-ascii 128)))) (response uint uint))
  
    (execute-call (uint (buff 2048)) (response bool uint))
    (execute-rollback (uint) (response bool uint))
    
    (verify-success (uint) (response bool uint))

    (handle-message ((string-ascii 128) (buff 2048)) (response bool uint))
    (handle-error (uint) (response bool uint))
    
    (set-admin (principal) (response bool uint))
    (set-protocol-fee-handler (principal) (response bool uint))
    (set-protocol-fee (uint) (response bool uint))
    (set-default-connection ((string-ascii 128) (string-ascii 128)) (response bool uint))
    (set-trusted-protocols ((string-ascii 128) (list 10 (string-ascii 128))) (response bool uint))
    
    (get-network-address () (response (string-ascii 257) uint))
    (get-network-id () (response (string-ascii 128) uint))
    (get-protocol-fee () (response uint uint))
    (get-fee ((string-ascii 128) bool (optional (list 10 (string-ascii 128)))) (response uint uint))
  )
)