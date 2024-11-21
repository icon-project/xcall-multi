(define-private (check_length (data (buff 4092)))
  (unwrap-panic (as-max-len? data u1024))
)

(define-read-only (encode-string (message (string-ascii 1024)))
   (let
       (   (encoded (unwrap-panic (to-consensus-buff? message)))
       )
       (if (is-eq (len encoded) u5)  ;; Empty string is type byte + 4 length bytes
           0x80
           (let (
               (encoded-length (- (len encoded) u5))
               (string-content (unwrap-panic (slice? encoded u5 (len encoded))))
           )
               (if (> encoded-length u55)
                   (let (
                       (length-bytes (unwrap-panic (to-consensus-buff? encoded-length)))
                       ;; Take last two bytes for length
                       (len-byte1 (unwrap-panic (element-at? length-bytes u15)))
                       (len-byte2 (unwrap-panic (element-at? length-bytes u16)))
                       (prefix (concat 0xb9 (concat len-byte1 len-byte2)))
                   )
                   (check_length (concat prefix string-content)))
                   (let (
                       (id (unwrap-panic (to-consensus-buff? (+ u128 encoded-length))))
                       (prefix (unwrap-panic (element-at? id u16)))
                       (res (concat prefix string-content))
                   )
                   (check_length res))
               )
           )
       )
   )
)

(define-private (encode-uint-raw (data uint))
    (if (is-eq data u0)
        0x80
        (let (
            (encoded (unwrap-panic (to-consensus-buff? data)))
            (sliced (unwrap-panic (slice? encoded u1 (len encoded))))
            (stripped (fold rm-lead sliced 0x00))
            )
            ;; For 4+ bytes, prefix 0x00 to avoid RLP misinterpreting first byte as string length
            (if (>= data (pow u256 u3))
                (check_length (concat 0x00 stripped))
                stripped)
        )
    )
)

(define-read-only (encode-uint (data uint))
    ;; 256^16 is the upper bound
    (if (>= data (pow u256 u15))
        (check_length (concat 0x91 (encode-uint-raw data)))
        (if (>= data (pow u256 u14))
            (check_length (concat 0x90 (encode-uint-raw data)))
            (if (>= data (pow u256 u13))
                (check_length (concat 0x8f (encode-uint-raw data)))
                (if (>= data (pow u256 u12))
                    (check_length (concat 0x8e (encode-uint-raw data)))
                    (if (>= data (pow u256 u11))
                        (check_length (concat 0x8d (encode-uint-raw data)))
                        (if (>= data (pow u256 u10))
                            (check_length (concat 0x8c (encode-uint-raw data)))
                            (if (>= data (pow u256 u9))
                                (check_length (concat 0x8b (encode-uint-raw data)))
                                (if (>= data (pow u256 u8))
                                    (check_length (concat 0x8a (encode-uint-raw data)))
                                    (if (>= data (pow u256 u7))
                                        (check_length (concat 0x89 (encode-uint-raw data)))
                                        (if (>= data (pow u256 u6))
                                            (check_length (concat 0x88 (encode-uint-raw data)))
                                            (if (>= data (pow u256 u5))
                                                (check_length (concat 0x87 (encode-uint-raw data)))
                                                (if (>= data (pow u256 u4))
                                                    (check_length (concat 0x86 (encode-uint-raw data)))
                                                    (if (>= data (pow u256 u3))
                                                        (check_length (concat 0x85 (encode-uint-raw data))) ;; We skip 0x84 for some reason
                                                        (if (>= data (pow u256 u2))
                                                            (check_length (concat 0x83 (encode-uint-raw data)))
                                                            (if (>= data u256)
                                                                (check_length (concat 0x82 (encode-uint-raw data)))
                                                                (if (>= data u128)
                                                                    (check_length (concat 0x81 (encode-uint-raw data)))
                                                                    (encode-uint-raw data)))))))))))))))))
)


(define-read-only  (encode-arr (objects (list 500 (buff 1024))))
  (encode-list-lenght (encode-buff-arr objects))
)

(define-private (encode-buff-arr (objects (list 500 (buff 1024))))
    (fold concat-buff objects 0x)
)

(define-private (encode-buff-long (data (buff 1024)))
  (let
        ((prefix (unwrap-panic  (element-at?  (unwrap-panic (to-consensus-buff? (+ u128 (len data))))  u16) )))
        (check_length (concat prefix data))
    )
)

(define-private (rm-lead (num (buff 1)) (buffer (buff 1024)))
    (if (is-eq 0x00 buffer)
        num
        (check_length (concat buffer num))
    )
)


(define-private (encode-list-lenght (data (buff 1024)))
 (let (
       (length (len data))
   )
   (if (<= length u55)
       (check_length (concat (encode-uint-raw (+ u192 length)) data))
       (let (
           (encoded-length (unwrap-panic (to-consensus-buff? length)))
           (stripped-length (unwrap-panic 
               (slice? encoded-length
                   (if (>= length u256) u14
                       (if (>= length u256) u15 u16))
                   (len encoded-length))))
           (prefix (encode-uint-raw (+ u247 (len stripped-length))))
       )
           (check_length (concat (concat prefix stripped-length) data))
       )
   )
))

(define-private (concat-buff (a (buff 1024)) (b (buff 1024)))
  (check_length (concat b a))
)