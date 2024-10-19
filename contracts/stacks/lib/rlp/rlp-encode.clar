(define-read-only  (encode-string (message (string-ascii 1024)))
    (let
        (   (encoded (unwrap-panic (to-consensus-buff? message)))
            (sliced  (unwrap-panic (slice? encoded u4  (len encoded))))
            (id (unwrap-panic  (to-consensus-buff? (+ u128 (buff-to-uint-le (unwrap-panic (element-at? sliced u0)))) )))
            (prefix (unwrap-panic  (element-at? id u16) ) )
            (res (replace-at? sliced u0 prefix ))
        )
        (check_length (unwrap-panic res))
    )
)

(define-read-only  (encode-uint (data uint))
  (encode-lenght (encode-uint-raw data))
)


(define-read-only  (encode-arr (objects (list 500 (buff 1024))))
  (encode-list-lenght (encode-buff-arr objects))
)

(define-private (encode-buff-arr (objects (list 500 (buff 1024))))
    (fold concat-buff objects 0x)
)

(define-read-only (encode-buff (data (buff 1024)))
    (if (< u1 (len data))
      (encode-buff-long data)
      data
    )
)

(define-private (rm-lead (num (buff 1)) (buffer (buff 1024)))
    (if (is-eq 0x00 buffer)
        num
        (check_length (concat buffer num))
    )
)

(define-private (encode-uint-raw (data uint))
    (let (
        (encoded (unwrap-panic (to-consensus-buff? data)))
        (sliced (unwrap-panic (slice? encoded u4  (len encoded))))
        )
        (check_length ( fold rm-lead sliced 0x00))
    )
)

(define-private (encode-lenght (data (buff 1024)))
  (let (
        (length (len data))
        )
        (if (<= length u1 )
            data
            (if (<= length u55 )
                (check_length (concat  (encode-uint-raw (+ u128 length)) data)) 
                (check_length (concat  (encode-uint-raw (+ u183 length)) data))
            )
        )
    )
)

(define-private (encode-list-lenght (data (buff 1024)))
  (let (
            (length (len data))
        )
        (if (<= length u55 )
            (check_length (concat  (encode-uint-raw (+ u192 length)) data))
            (let (
                    (encoded_lenght (encode-uint-raw length))
                    (prefix (concat (encode-uint-raw (+ u247 (len encoded_lenght))) encoded_lenght))
                )
                (check_length (concat  prefix data))
            )
        )
    )
)

(define-private (concat-buff (a (buff 1024)) (b (buff 1024)))
  (check_length (concat b a))
)

(define-private (check_length (data (buff 4092)))
  (unwrap-panic (as-max-len? data u1024))
)

(define-private (encode-buff-long (data (buff 1024)))
  (let
        ((prefix (unwrap-panic  (element-at?  (unwrap-panic (to-consensus-buff? (+ u128 (len data))))  u16) )))
        (check_length (concat prefix data))
    )
)