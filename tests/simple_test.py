import time
from donglao_g2p import Pipeline

pipeline = Pipeline()
texts = ["nice to meet you, tôi yêu bạn"] * 1

for i in range(0, 1):
    start = time.time()
    phonemes = pipeline.phonemize_batch(texts, normalize=False)
    print(phonemes)
    print("Time:\t", time.time() - start)

