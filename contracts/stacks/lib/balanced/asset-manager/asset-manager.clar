;; (use-trait ft-trait .sip-010-trait.sip-010-trait)
;; (impl-trait .xcall-receiver-trait.xcall-receiver-trait)

;; (define-constant CONTRACT_OWNER tx-sender)
;; (define-constant ICON_ASSET_MANAGER "0x1.icon/cxabea09a8c5f3efa54d0a0370b14715e6f2270591")
;; (define-constant X_CALL_NETWORK_ADDRESS "ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM.x-call")
;; (define-constant ERR_UNAUTHORIZED (err u100))
;; (define-constant ERR_INVALID_AMOUNT (err u101))
;; (define-constant ERR_EXCEED_WITHDRAW_LIMIT (err u102))
;; (define-constant ERR_INVALID_TOKEN (err u103))
;; (define-constant ERR_INVALID_MESSAGE (err u104))
;; (define-constant ERR_INVALID_MESSAGE_WITHDRAW_TO_NATIVE_UNSUPPORTED (err u105))
;; (define-constant POINTS u10000)
;; (define-constant NATIVE_TOKEN 'ST000000000000000000002AMW42H.nativetoken)
;; (define-constant SBTC_TOKEN 'ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM.sbtc)

;; (define-map limit-map principal {
;;   period: uint,
;;   percentage: uint,
;;   last-update: uint,
;;   current-limit: uint
;; })

;; (define-public (configure-rate-limit (token <ft-trait>) (new-period uint) (new-percentage uint))
;;   (begin
;;     (asserts! (is-eq tx-sender CONTRACT_OWNER) ERR_UNAUTHORIZED)
;;     (asserts! (<= new-percentage POINTS) ERR_INVALID_AMOUNT)
;;     (let ((balance (unwrap! (get-balance token) ERR_INVALID_AMOUNT)))
;;       (map-set limit-map (contract-of token) {
;;         period: new-period,
;;         percentage: new-percentage,
;;         last-update: block-height,
;;         current-limit: (/ (* balance new-percentage) POINTS)
;;       })
;;     )
;;     (ok true)
;;   )
;; )

;; (define-public (reset-limit (token <ft-trait>))
;;   (begin
;;     (asserts! (is-eq tx-sender CONTRACT_OWNER) ERR_UNAUTHORIZED)
;;     (let ((balance (unwrap-panic (get-balance token))))
;;       (let ((period-tuple (unwrap-panic (map-get? limit-map (contract-of token)))))
;;         (map-set limit-map (contract-of token) (merge period-tuple {
;;           current-limit: (/ (* balance (get percentage period-tuple)) POINTS)
;;         }))
;;       )
;;     )
;;     (ok true)
;;   )
;; )

;; (define-public (deposit-native (amount uint) )
;;   (begin
;;     (asserts! (> amount u0) ERR_INVALID_AMOUNT)
;;     (try! (stx-transfer? amount tx-sender (as-contract tx-sender)))
;;     ;; TODO: Send deposit message to ICON network
;;     (ok true)
;;   )
;; )

;; (define-public (deposit (token <ft-trait>) (amount uint))
;;   (begin
;;     (asserts! (> amount u0) ERR_INVALID_AMOUNT)
;;     (try! (contract-call? token transfer amount tx-sender (as-contract tx-sender) none))
;;     ;; TODO: Send deposit message to ICON network
;;     (ok true)
;;   )
;; )

;; (define-public (withdraw (token <ft-trait>) (amount uint) (recipient principal))
;;   (begin
;;     (asserts! (> amount u0) ERR_INVALID_AMOUNT)
;;     (let ((result (verify-withdraw token amount)))
;;       (if (is-ok result)
;;           (begin
;;             (try! (contract-call? token transfer amount (as-contract tx-sender) recipient none))
;;             (ok true)
;;           )
;;           (unwrap-err! result ERR_EXCEED_WITHDRAW_LIMIT) ;; is this throwing the right error?
;;       )
;;     )
;;   )
;; )

;; (define-public (handle-call-message (from (string-ascii 128)) (data (buff 2048)) (protocols (list 10 (string-ascii 128))))
;;   (let (
;;     (method-result (contract-call? .asset-manager-messages get-method data))
;;     (deposit-name (contract-call? .asset-manager-messages get-deposit-name))
;;     (deposit-revert-name (contract-call? .asset-manager-messages get-deposit-revert-name))
;;     (withdraw-to-name (contract-call? .asset-manager-messages get-withdraw-to-name))
;;     (withdraw-native-to-name (contract-call? .asset-manager-messages get-withdraw-native-to-name))
;;   )
;;     (asserts! (is-ok method-result) ERR_INVALID_MESSAGE)
;;     (let ((method (unwrap-panic method-result)))
;;       (if (is-eq method withdraw-to-name)
;;         (let ((message-result (contract-call? .asset-manager-messages decode-withdraw-to data)))
;;           (asserts! (is-ok message-result) ERR_INVALID_MESSAGE)
;;           (let ((message (unwrap-panic message-result)))
;;             (asserts! (is-eq from ICON_ASSET_MANAGER) ERR_UNAUTHORIZED)
;;             (let (
;;               (token-address-string (get token-address message))
;;               (to-address-string (get to message))
;;               (to-address-principal (contract-call? .util address-string-to-principal (unwrap-panic (as-max-len? to-address-string u128))))
;;               (amount (get amount message))
;;             )
;;               (if (is-eq token-address-string "ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM.sbtc")
;;                   (withdraw .sbtc amount (unwrap-panic to-address-principal))
;;                   ERR_INVALID_TOKEN
;;               )
;;             )
;;           )
;;         )
;;         (if (is-eq method withdraw-native-to-name)
;;           ERR_INVALID_MESSAGE_WITHDRAW_TO_NATIVE_UNSUPPORTED
;;           (if (is-eq method deposit-revert-name)
;;             (let ((message-result (contract-call? .asset-manager-messages decode-deposit-revert data)))
;;               (asserts! (is-ok message-result) ERR_INVALID_MESSAGE)
;;               (let ((message (unwrap-panic message-result)))
;;                 (asserts! (is-eq from X_CALL_NETWORK_ADDRESS) ERR_UNAUTHORIZED)
;;                 (let (
;;                   (token-address-string (get token-address message))
;;                   (to-address-string (get to message))
;;                   (to-address-principal (contract-call? .util address-string-to-principal (unwrap-panic (as-max-len? to-address-string u128))))
;;                   (amount (get amount message))
;;                 )
;;                   (if (is-eq token-address-string "ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM.sbtc")
;;                     (withdraw .sbtc amount (unwrap-panic to-address-principal))
;;                     ERR_INVALID_TOKEN
;;                   )
;;                 )
;;               )
;;             )
;;             ERR_INVALID_MESSAGE
;;           )
;;         )
;;       )
;;     )
;;   )
;; )

;; (define-read-only (get-current-limit (token <ft-trait>))
;;   (let ((period-tuple (unwrap-panic (map-get? limit-map (contract-of token)))))
;;     (get current-limit period-tuple)
;;   )
;; )

;; (define-read-only (get-period (token <ft-trait>))
;;   (let ((period-tuple (unwrap-panic (map-get? limit-map (contract-of token)))))
;;     (get period period-tuple)
;;   )
;; )

;; (define-read-only (get-percentage (token <ft-trait>))
;;   (let ((period-tuple (unwrap-panic (map-get? limit-map (contract-of token)))))
;;     (get percentage period-tuple)
;;   )
;; )

;; (define-private (get-balance (token <ft-trait>))
;;   (if (is-eq (contract-of token) NATIVE_TOKEN)
;;       (ok (stx-get-balance (as-contract tx-sender)))
;;       (ok (unwrap! (contract-call? token get-balance (as-contract tx-sender)) ERR_INVALID_AMOUNT))
;;   )
;; )

;; (define-private (verify-withdraw (token <ft-trait>) (amount uint))
;;   (let ((balance (unwrap-panic (get-balance token))))
;;     (let ((limit (calculate-limit balance token)))
;;       (if (< amount limit)
;;           (begin
;;             (let ((period-tuple (unwrap-panic (map-get? limit-map (contract-of token)))))
;;               (map-set limit-map (contract-of token) (merge period-tuple {
;;                 current-limit: (- limit amount),
;;                 last-update: block-height
;;               }))
;;             )
;;             (ok true)
;;           )
;;           (err ERR_EXCEED_WITHDRAW_LIMIT)
;;       )
;;     )
;;   )
;; )

;; (define-private (calculate-limit (balance uint) (token <ft-trait>))
;;   (let ((period-tuple (unwrap-panic (map-get? limit-map (contract-of token)))))
;;     (let ((token-period (get period period-tuple)))
;;       (let ((token-percentage (get percentage period-tuple)))
;;         (let ((max-limit (/ (* balance token-percentage) POINTS)))
;;           (let ((max-withdraw (- balance max-limit)))
;;             (let ((time-diff (- block-height (get last-update period-tuple))))
;;               (let ((capped-time-diff (if (< time-diff token-period) time-diff token-period)))
;;                 (let ((added-allowed-withdrawal (/ (* max-withdraw capped-time-diff) token-period)))
;;                   (let ((limit (+ (get current-limit period-tuple) added-allowed-withdrawal)))
;;                     (let ((capped-limit (if (< balance limit) balance limit)))
;;                       (if (> capped-limit max-limit) max-limit capped-limit)
;;                     )
;;                   )
;;                 )
;;               )
;;             )
;;           )
;;         )
;;       )
;;     )
;;   )
;; )
