
;; title: asset-manager-messages
;; version:
;; summary:
;; description:

;; traits
;;

;; token definitions
;;

;; constants
(define-constant DEPOSIT_NAME "Deposit")
(define-constant DEPOSIT_REVERT_NAME "DepositRevert")
(define-constant WITHDRAW_TO_NAME "WithdrawTo")
(define-constant WITHDRAW_NATIVE_TO_NAME "WithdrawNativeTo")

(define-constant ERR_INVALID_METHOD (err u100))
;;

;; data vars
;;

;; data maps
;; messages are stored in memory. these are not called, but written to define the struct
(define-map Deposit uint {
  tokenAddress: (string-ascii 500),
  from: (string-ascii 500),
  to: (string-ascii 500),
  amount: uint,
  data: (buff 500)
}
)

(define-map DepositRevert uint {
  tokenAddress: (string-ascii 500),
  amount: uint,
  to: (string-ascii 500)
}
)

(define-map WithdrawTo uint {
  tokenAddress: (string-ascii 500),
  to: (string-ascii 500),
  amount: uint
}
)
;;

;; public functions
(define-public (encode-deposit (message (tuple (tokenAddress (string-ascii 500)) (from (string-ascii 500)) (to (string-ascii 500)) (amount uint) (data (buff 500)))))
  (let (
    (token-address (get tokenAddress message))
    (from (get from message))
    (to (get to message))
    (amount (get amount message))
    (data (get data message))
  )
    (ok (contract-call? .rlp-encode encode-arr
      (list
        (contract-call? .rlp-encode encode-string DEPOSIT_NAME)
        (contract-call? .rlp-encode encode-string token-address)
        (contract-call? .rlp-encode encode-string from)
        (contract-call? .rlp-encode encode-string to)
        (contract-call? .rlp-encode encode-uint amount)
        (contract-call? .rlp-encode encode-buff data)
      )
    ))
  )
)

(define-public (encode-deposit-revert (message (tuple (tokenAddress (string-ascii 500)) (amount uint) (to (string-ascii 500)))))
  (let (
    (token-address (get tokenAddress message))
    (amount (get amount message))
    (to (get to message))
  )
    (ok (contract-call? .rlp-encode encode-arr
      (list
        (contract-call? .rlp-encode encode-string DEPOSIT_REVERT_NAME)
        (contract-call? .rlp-encode encode-string token-address)
        (contract-call? .rlp-encode encode-uint amount)
        (contract-call? .rlp-encode encode-string to)
      )
    ))
  )
)

(define-public (encode-withdraw-to (message (tuple (tokenAddress (string-ascii 500)) (to (string-ascii 500)) (amount uint))))
  (let (
    (token-address (get tokenAddress message))
    (to (get to message))
    (amount (get amount message))
  )
    (ok (contract-call? .rlp-encode encode-arr
      (list
        (contract-call? .rlp-encode encode-string WITHDRAW_TO_NAME)
        (contract-call? .rlp-encode encode-string token-address)
        (contract-call? .rlp-encode encode-string to)
        (contract-call? .rlp-encode encode-uint amount)
      )
    ))
  )
)

(define-public (encode-withdraw-native-to (message (tuple (tokenAddress (string-ascii 500)) (to (string-ascii 500)) (amount uint))))
  (let (
    (token-address (get tokenAddress message))
    (to (get to message))
    (amount (get amount message))
  )
    (ok (contract-call? .rlp-encode encode-arr
      (list
        (contract-call? .rlp-encode encode-string WITHDRAW_NATIVE_TO_NAME)
        (contract-call? .rlp-encode encode-string token-address)
        (contract-call? .rlp-encode encode-string to)
        (contract-call? .rlp-encode encode-uint amount)
      )
    ))
  )
)
;;

;; read only functions
(define-read-only (get-method (data (buff 1024)))
  (let (
    (rlp-list (contract-call? .rlp-decode rlp-to-list data))
    (method-bytes (contract-call? .rlp-decode rlp-decode-string rlp-list u0))
  )
    (if (is-eq method-bytes DEPOSIT_NAME)
        (ok DEPOSIT_NAME)
        (if (is-eq method-bytes DEPOSIT_REVERT_NAME)
            (ok DEPOSIT_REVERT_NAME)
            (if (is-eq method-bytes WITHDRAW_TO_NAME)
                (ok WITHDRAW_TO_NAME)
                (if (is-eq method-bytes WITHDRAW_NATIVE_TO_NAME)
                    (ok WITHDRAW_NATIVE_TO_NAME)
                    ERR_INVALID_METHOD
                )
            )
        )
    )
  )
)

(define-read-only (get-deposit-name)
  DEPOSIT_NAME
)

(define-read-only (get-deposit-revert-name)
  DEPOSIT_REVERT_NAME
)

(define-read-only (get-withdraw-to-name)
  WITHDRAW_TO_NAME
)

(define-read-only (get-withdraw-native-to-name)
  WITHDRAW_NATIVE_TO_NAME
)

(define-read-only (decode-withdraw-to (data (buff 1024)))
  (let (
    (rlp-list (contract-call? .rlp-decode rlp-to-list data))
    (token-address (contract-call? .rlp-decode rlp-decode-string rlp-list u1))
    (to (contract-call? .rlp-decode rlp-decode-string rlp-list u2))
    (amount (contract-call? .rlp-decode rlp-decode-uint rlp-list u3))
  )
    (ok (tuple (token-address token-address) (to to) (amount amount)))
  )
)

(define-read-only (decode-deposit-revert (data (buff 1024)))
  (let (
    (rlp-list (contract-call? .rlp-decode rlp-to-list data))
    (token-address (contract-call? .rlp-decode rlp-decode-string rlp-list u1))
    (amount (contract-call? .rlp-decode rlp-decode-uint rlp-list u2))
    (to (contract-call? .rlp-decode rlp-decode-string rlp-list u3))
  )
    (ok (tuple (token-address token-address) (amount amount) (to to)))
  )
)
;;

;; private functions
;;

