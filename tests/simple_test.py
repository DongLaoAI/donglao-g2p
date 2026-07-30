import time
from donglao_g2p import Pipeline

pipeline = Pipeline()
texts = ["Mã nguồn Nam Á Bank là gì."] * 1

for i in range(0, 10):
    start = time.time()
    phonemes = pipeline.phonemize_batch(texts, normalize=True)
    print(phonemes)
    print("Time:\t", time.time() - start)

