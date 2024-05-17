(import cv2)
(import numpy :as np)

(defmacro defconst [name value]
  `(do
     (defn ~name [] ~value)))

(defconst PHASH-SZ 8)

(defn showAndWait [img] 
  (cv2.imshow "Preview" img)
  (cv2.waitKey))

(defn phash [img]
  (setv img 
    (cv2.resize img #((PHASH-SZ) (PHASH-SZ)) 
                :interpolation cv2.INTER_LINEAR)) 
  (setv gray 
    (cv2.cvtColor img cv2.COLOR_BGR2GRAY)) 
  (setv average
    (int (np.average gray))) 
  (setv [ret binary]
    (cv2.threshold gray average 1 cv2.THRESH_BINARY)) 
  (.reshape binary #(-1)))

(defn calc-sim [ph1 ph2]
  (setv pix
       (sum (map (fn [ v ] (= (get v 1) (get v 0))) (zip ph1 ph2)))) 
  (setv sz 
    (min (len ph1) (len ph2))) 
  (/ pix sz))
  

(defn main []
  (setv lena (cv2.imread "./lena.png"))
  (setv p1 (phash lena))
  (setv p2 (phash lena))
  (print p1)
  (print p2)
  (print (calc-sim p1 p2)))

(main)