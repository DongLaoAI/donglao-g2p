import time
from donglao_g2p import Pipeline

pipeline = Pipeline(language='auto')
texts = ["nice to meet you mã nguồn Nam Á bank hello, là gì"] * 1

for i in range(0, 10):
    start = time.time()
    phonemes = pipeline.phonemize_batch(texts, normalize=True)
    print(phonemes)
    print("Time:\t", time.time() - start)

# uv sync --dev
# uv run pytest tests/test_simple.py