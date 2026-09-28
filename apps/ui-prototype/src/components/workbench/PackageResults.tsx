import type { CheckLocation } from "../../check-types";
import type { PackageResult, PackageProgram } from "../../package-types";
const PAGE = 12;
const size = (bytes: number) =>
  bytes < 1024 ? `${bytes} 字节` : `${(bytes / 1024).toFixed(1)} KiB`;
export function PackageResults({
  result,
  reportPrograms,
  current,
  disabled,
  resultPage,
  setResultPage,
  onLocate,
}: {
  result: PackageResult;
  reportPrograms: PackageProgram[];
  current: boolean;
  disabled: boolean;
  resultPage: number;
  setResultPage(page: number): void;
  onLocate(location: CheckLocation, generation: number): Promise<boolean>;
}) {
  const report = result.report;
  const resultCount = report?.programs.length ?? result.issues.length;
  const resultPages = Math.max(1, Math.ceil(resultCount / PAGE));
  const activeResultPage = Math.min(resultPage, resultPages - 1);
  return (
    <>
      {report && (
        <>
          <div className="wb-package-metrics">
            <span>
              <b>{report.programs.length}</b> 个节目
            </span>
            <span>
              <b>{size(report.bytes)}</b> 文件大小
            </span>
            <span>
              <b>
                {size(
                  Math.max(...report.programs.map((p) => p.loaderPeakBytes)),
                )}
              </b>{" "}
              最大参考装载峰值 / {size(report.maxLoaderBytes)}
            </span>
          </div>
          <div className="wb-package-results">
            {reportPrograms
              .slice(activeResultPage * PAGE, (activeResultPage + 1) * PAGE)
              .map((p, i) => (
                <div key={activeResultPage * PAGE + i}>
                  <span>
                    <b>{p.name}</b>
                    <small>
                      {p.steps} 步 · {p.attributes} 个属性 · {p.effectChannels}{" "}
                      个效果通道
                    </small>
                  </span>
                  <span>
                    {size(p.encodedBytes)}
                    <small>参考装载 {size(p.loaderPeakBytes)}</small>
                  </span>
                  <button
                    disabled={!current || disabled}
                    onClick={() => void onLocate(p.location, result.generation)}
                  >
                    定位
                  </button>
                </div>
              ))}
          </div>
          <details className="wb-package-details">
            <summary>文件校验信息</summary>
            <p>来源快照：{report.sourceDigest}</p>
            <p>播放包：{report.packageDigest}</p>
            <p>
              输出线路 {report.universe} · 目录参考内存{" "}
              {size(report.catalogResidentBytes)}
            </p>
          </details>
        </>
      )}
      {!!result?.issues.length && (
        <div className="wb-package-results" aria-label="播放包问题">
          {result.issues
            .slice(activeResultPage * PAGE, (activeResultPage + 1) * PAGE)
            .map((issue, i) => (
              <div key={i}>
                <span>{issue.message}</span>
                {issue.location && (
                  <button
                    disabled={!current || disabled}
                    onClick={() =>
                      void onLocate(issue.location!, result.generation)
                    }
                  >
                    定位问题
                  </button>
                )}
              </div>
            ))}
        </div>
      )}
      {resultPages > 1 && (
        <div className="wb-package-pager">
          <button
            disabled={activeResultPage === 0}
            onClick={() => setResultPage(activeResultPage - 1)}
          >
            上一组结果
          </button>
          <span>
            结果 {activeResultPage + 1} / {resultPages} 页
          </span>
          <button
            disabled={activeResultPage + 1 >= resultPages}
            onClick={() => setResultPage(activeResultPage + 1)}
          >
            下一组结果
          </button>
        </div>
      )}
    </>
  );
}
